use crate::geometry::{Point, Vector3};
use crate::hexasphere::{Hexasphere, ShapeAnalyzer};
use serde::{Deserialize, Serialize};

/// A unique hexagon shape template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeTemplate {
    /// Unique identifier for this shape
    pub shape_id: usize,
    /// Vertices relative to origin (normalized shape)
    pub vertices: Vec<[f64; 3]>,
    /// How many tiles use this shape
    pub instance_count: usize,
}

/// Instance of a shape at a specific location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeInstance {
    /// Which shape template to use
    pub shape_id: usize,
    /// Center position of the tile
    pub center: [f64; 3],
    /// Rotation quaternion [x, y, z, w] to orient the shape
    pub rotation: [f64; 4],
    /// Scale factor (usually 1.0)
    pub scale: f64,
}

/// Hexasphere data optimized for instanced rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstancedHexasphere {
    /// Unique shape templates
    pub shapes: Vec<ShapeTemplate>,
    /// All tile instances with their transformations
    pub instances: Vec<ShapeInstance>,
    /// Sphere radius
    pub radius: f64,
    /// Number of subdivisions used
    pub subdivisions: u32,
}

impl Hexasphere {
    /// Export hexasphere data optimized for instanced rendering
    pub fn export_instanced(&self, subdivisions: u32) -> InstancedHexasphere {
        let tiles = &self.tiles;
        let centers: Vec<_> = tiles.iter().map(|t| t.center_point.clone()).collect();

        let mut analyzer = ShapeAnalyzer::new();
        analyzer.analyze_tiles(tiles, &centers);

        let shape_indices = analyzer.get_shape_indices();
        let representatives = analyzer.get_shape_representatives();

        // Create shape templates
        let mut shapes = Vec::new();
        let mut shape_counts = vec![0; representatives.len()];

        // Count instances per shape
        for &shape_idx in &shape_indices {
            shape_counts[shape_idx] += 1;
        }

        // Build shape templates from representatives
        for (shape_id, &tile_idx) in representatives.iter().enumerate() {
            let tile = &tiles[tile_idx];
            let center = &centers[tile_idx];

            // Convert tile vertices to relative coordinates
            let vertices: Vec<[f64; 3]> = tile
                .boundary
                .iter()
                .map(|p| {
                    let relative = Vector3::new(p.x - center.x, p.y - center.y, p.z - center.z);
                    [relative.x, relative.y, relative.z]
                })
                .collect();

            shapes.push(ShapeTemplate {
                shape_id,
                vertices,
                instance_count: shape_counts[shape_id],
            });
        }

        // Build instances with transformations
        let mut instances = Vec::new();

        for (tile_idx, tile) in tiles.iter().enumerate() {
            let shape_id = shape_indices[tile_idx];
            let center = &centers[tile_idx];

            // Calculate rotation to align template with actual tile
            let rotation = calculate_rotation_to_tile(
                &tiles[representatives[shape_id]],
                &centers[representatives[shape_id]],
                tile,
                center,
            );

            instances.push(ShapeInstance {
                shape_id,
                center: [center.x, center.y, center.z],
                rotation,
                scale: 1.0,
            });
        }

        InstancedHexasphere {
            shapes,
            instances,
            radius: self.radius,
            subdivisions,
        }
    }
}

/// Calculate rotation quaternion to align template shape with target tile
fn calculate_rotation_to_tile(
    template_tile: &crate::tile::Tile,
    template_center: &Point,
    target_tile: &crate::tile::Tile,
    target_center: &Point,
) -> [f64; 4] {
    // Get the "up" direction (normal) for both tiles
    let template_up =
        Vector3::new(template_center.x, template_center.y, template_center.z).normalize();

    let target_up = Vector3::new(target_center.x, target_center.y, target_center.z).normalize();

    // If tiles have the same number of vertices, try to align edges
    if template_tile.boundary.len() == target_tile.boundary.len() {
        // Find the longest edge in template
        let template_edge = find_longest_edge_direction(template_tile, template_center);
        let target_edge = find_longest_edge_direction(target_tile, target_center);

        // Calculate rotation that aligns both up vectors and edge directions
        rotation_between_frames(template_up, template_edge, target_up, target_edge)
    } else {
        // Just align the up vectors
        rotation_between_vectors(template_up, target_up)
    }
}

/// Find direction of longest edge from center
fn find_longest_edge_direction(tile: &crate::tile::Tile, center: &Point) -> Vector3 {
    let mut longest_length = 0.0;
    let mut longest_dir = Vector3::new(1.0, 0.0, 0.0);

    for i in 0..tile.boundary.len() {
        let next = (i + 1) % tile.boundary.len();
        let edge_length = tile.boundary[i].distance_to(&tile.boundary[next]);

        if edge_length > longest_length {
            longest_length = edge_length;
            // Direction from center to midpoint of longest edge
            let midpoint = Point::new(
                (tile.boundary[i].x + tile.boundary[next].x) / 2.0,
                (tile.boundary[i].y + tile.boundary[next].y) / 2.0,
                (tile.boundary[i].z + tile.boundary[next].z) / 2.0,
            );
            longest_dir = Vector3::new(
                midpoint.x - center.x,
                midpoint.y - center.y,
                midpoint.z - center.z,
            )
            .normalize();
        }
    }

    longest_dir
}

/// Calculate quaternion rotation between two vectors
fn rotation_between_vectors(from: Vector3, to: Vector3) -> [f64; 4] {
    let axis = from.cross(&to);
    let angle = from.angle_to(&to);

    if axis.magnitude() < 1e-6 {
        // Vectors are parallel or anti-parallel
        if from.dot(&to) > 0.0 {
            // Same direction
            [0.0, 0.0, 0.0, 1.0]
        } else {
            // Opposite direction - rotate 180 degrees around any perpendicular axis
            let perp = if from.x.abs() < 0.9 {
                Vector3::new(1.0, 0.0, 0.0)
            } else {
                Vector3::new(0.0, 1.0, 0.0)
            };
            let axis = from.cross(&perp).normalize();
            [axis.x, axis.y, axis.z, 0.0]
        }
    } else {
        // Normal case
        let axis = axis.normalize();
        let half_angle = angle / 2.0;
        let s = half_angle.sin();
        [axis.x * s, axis.y * s, axis.z * s, half_angle.cos()]
    }
}

/// Calculate quaternion rotation between two coordinate frames
fn rotation_between_frames(
    from_up: Vector3,
    from_forward: Vector3,
    to_up: Vector3,
    to_forward: Vector3,
) -> [f64; 4] {
    // First rotate to align up vectors
    let up_rotation = rotation_between_vectors(from_up, to_up);

    // Apply this rotation to the forward vector
    let rotated_forward = rotate_vector_by_quaternion(from_forward, up_rotation);

    // Project both forward vectors onto the plane perpendicular to up
    let projected_from = project_onto_plane(rotated_forward, to_up);
    let projected_to = project_onto_plane(to_forward, to_up);

    if projected_from.magnitude() < 1e-6 || projected_to.magnitude() < 1e-6 {
        // One of the vectors is parallel to up, just use up rotation
        up_rotation
    } else {
        // Calculate additional rotation around up axis
        let forward_rotation = rotation_around_axis(
            to_up,
            projected_from
                .normalize()
                .angle_to(&projected_to.normalize()),
        );

        // Combine rotations
        quaternion_multiply(forward_rotation, up_rotation)
    }
}

/// Rotate a vector by a quaternion
fn rotate_vector_by_quaternion(v: Vector3, q: [f64; 4]) -> Vector3 {
    let [qx, qy, qz, qw] = q;

    // Quaternion multiplication: q * v * q^(-1)
    let ix = qw * v.x + qy * v.z - qz * v.y;
    let iy = qw * v.y + qz * v.x - qx * v.z;
    let iz = qw * v.z + qx * v.y - qy * v.x;
    let iw = -qx * v.x - qy * v.y - qz * v.z;

    Vector3::new(
        ix * qw - iw * qx - iy * qz + iz * qy,
        iy * qw - iw * qy - iz * qx + ix * qz,
        iz * qw - iw * qz - ix * qy + iy * qx,
    )
}

/// Project vector onto plane perpendicular to normal
fn project_onto_plane(v: Vector3, normal: Vector3) -> Vector3 {
    let n = normal.normalize();
    let dot = v.dot(&n);
    Vector3::new(v.x - dot * n.x, v.y - dot * n.y, v.z - dot * n.z)
}

/// Create quaternion for rotation around axis
fn rotation_around_axis(axis: Vector3, angle: f64) -> [f64; 4] {
    let axis = axis.normalize();
    let half_angle = angle / 2.0;
    let s = half_angle.sin();
    [axis.x * s, axis.y * s, axis.z * s, half_angle.cos()]
}

/// Multiply two quaternions
fn quaternion_multiply(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    let [ax, ay, az, aw] = a;
    let [bx, by, bz, bw] = b;

    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instanced_export() {
        let hs = Hexasphere::new(1.0, 3, 1.0);
        let instanced = hs.export_instanced(3);

        println!("Shapes: {}", instanced.shapes.len());
        println!("Instances: {}", instanced.instances.len());

        // Should have significantly fewer shapes than instances
        assert!(instanced.shapes.len() < instanced.instances.len() / 5);

        // All instances should reference valid shapes
        for instance in &instanced.instances {
            assert!(instance.shape_id < instanced.shapes.len());
        }
    }
}
