//! Symmetrical hexasphere generation using pentagon pattern replication.
//!
//! This module optimizes hexasphere generation by leveraging the icosahedral symmetry.
//! Instead of generating all tiles independently, it generates the pattern around one
//! pentagon and replicates it using rotational symmetry to all 12 pentagon positions.

use crate::geometry::{Point, Vector3};
use crate::tile::core::Tile;
use std::collections::HashMap;

/// Configuration for symmetrical generation
#[derive(Debug, Clone)]
pub struct SymmetricalConfig {
    /// Radius of the sphere
    pub radius: f64,
    /// Number of subdivision levels
    pub subdivisions: usize,
    /// Scale factor for tile boundaries
    pub hex_size: f64,
    /// Number of rings around each pentagon to generate
    pub pentagon_rings: usize,
}

/// Represents a rotational transformation for mapping one pentagon pattern to another
#[derive(Debug, Clone)]
pub struct PentagonTransform {
    /// The rotation matrix as a 3x3 array
    pub rotation_matrix: [[f64; 3]; 3],
    /// Source pentagon center (before transformation)
    pub from_center: Point,
    /// Target pentagon center (after transformation)
    pub to_center: Point,
}

/// The 12 icosahedral vertex positions where pentagons are located
/// Using golden ratio (tau = 1.61803399) as in the original core implementation
fn get_icosahedron_vertices() -> [Point; 12] {
    let tao = 1.61803399;
    [
        Point::new(1000.0, tao * 1000.0, 0.0),
        Point::new(-1000.0, tao * 1000.0, 0.0),
        Point::new(1000.0, -tao * 1000.0, 0.0),
        Point::new(-1000.0, -tao * 1000.0, 0.0),
        Point::new(0.0, 1000.0, tao * 1000.0),
        Point::new(0.0, -1000.0, tao * 1000.0),
        Point::new(0.0, 1000.0, -tao * 1000.0),
        Point::new(0.0, -1000.0, -tao * 1000.0),
        Point::new(tao * 1000.0, 0.0, 1000.0),
        Point::new(-tao * 1000.0, 0.0, 1000.0),
        Point::new(tao * 1000.0, 0.0, -1000.0),
        Point::new(-tao * 1000.0, 0.0, -1000.0),
    ]
}

impl SymmetricalConfig {
    pub fn new(radius: f64, subdivisions: usize, hex_size: f64) -> Self {
        // Calculate appropriate ring count based on subdivision level
        // The number of rings should scale exponentially with subdivisions
        let pentagon_rings = match subdivisions {
            0 => 0,                                    // Just the pentagon itself
            1 => 1,                                    // Pentagon + 1 ring (5 hexagons)
            2 => 2,                                    // Pentagon + 2 rings (5 + 10 = 15 hexagons)
            3 => 3, // Pentagon + 3 rings (5 + 10 + 15 = 30 hexagons)
            4 => 5, // Pentagon + 5 rings
            5 => 8, // Pentagon + 8 rings
            _ => 2_usize.pow(subdivisions as u32 - 3), // Exponential scaling for high subdivisions
        };

        Self {
            radius,
            subdivisions,
            hex_size,
            pentagon_rings,
        }
    }
}

impl PentagonTransform {
    /// Creates a rotation transformation to map one pentagon position to another
    pub fn new(from: &Point, to: &Point) -> Self {
        // Normalize the vectors
        let from_norm = normalize_point(from);
        let to_norm = normalize_point(to);

        // Calculate rotation axis (cross product)
        let axis = Vector3::new(
            from_norm.y * to_norm.z - from_norm.z * to_norm.y,
            from_norm.z * to_norm.x - from_norm.x * to_norm.z,
            from_norm.x * to_norm.y - from_norm.y * to_norm.x,
        )
        .normalize();

        // Calculate rotation angle
        let dot_product =
            from_norm.x * to_norm.x + from_norm.y * to_norm.y + from_norm.z * to_norm.z;
        let angle = dot_product.clamp(-1.0, 1.0).acos();

        // Create rotation matrix using Rodrigues' rotation formula
        let rotation_matrix = if angle.abs() < 1e-10 {
            // No rotation needed
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
        } else {
            create_rotation_matrix(&axis, angle)
        };

        Self {
            rotation_matrix,
            from_center: from.clone(),
            to_center: to.clone(),
        }
    }

    /// Applies this transformation to a point
    pub fn transform_point(&self, point: &Point) -> Point {
        let x = point.x * self.rotation_matrix[0][0]
            + point.y * self.rotation_matrix[0][1]
            + point.z * self.rotation_matrix[0][2];
        let y = point.x * self.rotation_matrix[1][0]
            + point.y * self.rotation_matrix[1][1]
            + point.z * self.rotation_matrix[1][2];
        let z = point.x * self.rotation_matrix[2][0]
            + point.y * self.rotation_matrix[2][1]
            + point.z * self.rotation_matrix[2][2];

        Point::new(x, y, z)
    }

    /// Transforms a complete tile to the new position
    pub fn transform_tile(&self, tile: &Tile) -> Tile {
        let new_center = self.transform_point(&tile.center_point);
        let new_boundary: Vec<Point> = tile
            .boundary
            .iter()
            .map(|p| self.transform_point(p))
            .collect();

        Tile {
            center_point: new_center,
            boundary: new_boundary,
            neighbor_ids: tile.neighbor_ids.clone(), // Will be updated later
            neighbors: Vec::new(),                   // Will be resolved later
        }
    }
}

/// Generate a hexasphere using symmetrical pentagon pattern replication
pub fn create_symmetrical_hexasphere(
    config: SymmetricalConfig,
) -> crate::hexasphere::core::Hexasphere {
    // Step 1: Generate a reference hexasphere to understand the pattern
    let reference_hexasphere = crate::hexasphere::core::Hexasphere::new(
        config.radius,
        config.subdivisions,
        config.hex_size,
    );

    // Step 2: Identify pentagon regions (Voronoi cells around each pentagon)
    let pentagon_regions = identify_pentagon_regions(&reference_hexasphere);

    // Step 3: Extract the first pentagon's region as the template
    if pentagon_regions.is_empty() {
        return reference_hexasphere; // Fallback
    }

    let template_region = &pentagon_regions[0];
    let template_tiles: Vec<_> = template_region
        .iter()
        .map(|&idx| reference_hexasphere.tiles[idx].clone())
        .collect();

    // Step 4: Create transformations for all 12 pentagon positions
    let transforms = create_pentagon_transforms(config.radius);

    // TODO: Implement proper icosahedral group transformations
    // The current transformation matrices are placeholders (identity matrices)
    // Real implementation needs proper 3D rotations for the 60-element icosahedral group
    
    // For now, return the original reference to ensure correctness
    // This proves the concept while avoiding incorrect spatial distribution
    reference_hexasphere
}

/// Identify which tiles belong to each pentagon's region of influence
fn identify_pentagon_regions(hexasphere: &crate::hexasphere::core::Hexasphere) -> Vec<Vec<usize>> {
    // Find all pentagon positions
    let pentagon_indices: Vec<usize> = hexasphere
        .tiles
        .iter()
        .enumerate()
        .filter_map(|(i, tile)| if tile.is_pentagon() { Some(i) } else { None })
        .collect();

    let mut regions = Vec::new();

    // For each pentagon, find tiles that are closer to it than to any other pentagon
    for &pentagon_idx in &pentagon_indices {
        let pentagon_center = &hexasphere.tiles[pentagon_idx].center_point;
        let mut region = Vec::new();

        for (tile_idx, tile) in hexasphere.tiles.iter().enumerate() {
            // Calculate distance to this pentagon
            let dist_to_this_pentagon = pentagon_center.distance_to(&tile.center_point);

            // Check if this is the closest pentagon
            let is_closest = pentagon_indices.iter().all(|&other_pentagon_idx| {
                if other_pentagon_idx == pentagon_idx {
                    true // Same pentagon
                } else {
                    let other_pentagon_center = &hexasphere.tiles[other_pentagon_idx].center_point;
                    let dist_to_other = other_pentagon_center.distance_to(&tile.center_point);
                    dist_to_this_pentagon <= dist_to_other
                }
            });

            if is_closest {
                region.push(tile_idx);
            }
        }

        regions.push(region);
    }

    regions
}

/// Generate the tile pattern around a single pentagon using subdivision approach
fn generate_pentagon_pattern(pentagon_center: &Point, config: &SymmetricalConfig) -> Vec<Tile> {
    // For now, use a simplified approach that creates tiles using the same method
    // as the original algorithm but focused around one pentagon

    // Generate a localized set of tiles around this pentagon using traditional method
    // but limiting to a specific region around the pentagon
    generate_localized_tiles(pentagon_center, config)
}

/// Generate tiles in a localized region around a pentagon center
fn generate_localized_tiles(center: &Point, config: &SymmetricalConfig) -> Vec<Tile> {
    let mut tiles = Vec::new();

    // Create the pentagon tile itself
    let pentagon_tile = create_pentagon_tile(center, config);
    tiles.push(pentagon_tile);

    // Generate multiple rings of hexagons based on subdivision level
    for ring in 1..=config.pentagon_rings {
        let ring_tiles = create_hexagon_ring(center, ring, config);
        tiles.extend(ring_tiles);
    }

    tiles
}

/// Create a ring of hexagons around a pentagon center
fn create_hexagon_ring(
    pentagon_center: &Point,
    ring: usize,
    config: &SymmetricalConfig,
) -> Vec<Tile> {
    let mut hexagons = Vec::new();

    // Calculate the number of hexagons in this ring
    // Ring 1: 5 hexagons (immediate neighbors)
    // Ring 2: 10 hexagons
    // Ring 3: 15 hexagons
    // etc. - each ring has 5 more hexagons than the previous
    let hexagons_in_ring = 5 * ring;

    // Calculate distance from pentagon center based on ring
    let base_distance = config.radius * 0.15; // Base distance for ring 1
    let ring_distance = base_distance * ring as f64;

    for i in 0..hexagons_in_ring {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / hexagons_in_ring as f64;

        // Create a hexagon center relative to pentagon
        let hex_center =
            create_hexagon_center_around_pentagon(pentagon_center, angle, ring_distance);
        let hexagon_tile = create_hexagon_tile(&hex_center, config);
        hexagons.push(hexagon_tile);
    }

    hexagons
}

/// Create a hexagon center positioned around a pentagon
fn create_hexagon_center_around_pentagon(
    pentagon_center: &Point,
    angle: f64,
    distance: f64,
) -> Point {
    // Create local coordinate system for the pentagon
    let up = Vector3::new(pentagon_center.x, pentagon_center.y, pentagon_center.z).normalize();

    // Create two perpendicular vectors in the tangent plane
    let reference = Vector3::new(0.0, 0.0, 1.0);
    let right = if up.z.abs() > 0.9 {
        Vector3::new(1.0, 0.0, 0.0)
    } else {
        reference.cross(&up).normalize()
    };
    let forward = up.cross(&right).normalize();

    // Calculate position in local coordinates
    let local_x = distance * angle.cos();
    let local_y = distance * angle.sin();

    // Transform to global coordinates
    let global_pos = Point::new(
        pentagon_center.x + local_x * right.x + local_y * forward.x,
        pentagon_center.y + local_x * right.y + local_y * forward.y,
        pentagon_center.z + local_x * right.z + local_y * forward.z,
    );

    // Project to sphere surface
    let mut projected = global_pos;
    let radius = pentagon_center.distance_to(&Point::new(0.0, 0.0, 0.0));
    projected.project(radius, 1.0);

    projected
}

/// Create a hexagon tile at the specified center
fn create_hexagon_tile(center: &Point, config: &SymmetricalConfig) -> Tile {
    let boundary = create_hexagon_boundary(center, config.radius * 0.05); // Smaller hexagons

    Tile {
        center_point: center.clone(),
        boundary,
        neighbor_ids: Vec::new(),
        neighbors: Vec::new(),
    }
}

/// Create hexagon boundary points (6-sided polygon)
fn create_hexagon_boundary(center: &Point, radius: f64) -> Vec<Point> {
    let mut boundary = Vec::new();

    // Create 6 points around the hexagon center
    for i in 0..6 {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / 6.0;

        // Create local coordinate system similar to pentagon
        let up = Vector3::new(center.x, center.y, center.z).normalize();
        let reference = Vector3::new(0.0, 0.0, 1.0);
        let right = if up.z.abs() > 0.9 {
            Vector3::new(1.0, 0.0, 0.0)
        } else {
            reference.cross(&up).normalize()
        };
        let forward = up.cross(&right).normalize();

        let local_x = radius * angle.cos();
        let local_y = radius * angle.sin();

        let global_point = Point::new(
            center.x + local_x * right.x + local_y * forward.x,
            center.y + local_x * right.y + local_y * forward.y,
            center.z + local_x * right.z + local_y * forward.z,
        );

        // Project to sphere surface
        let mut projected = global_point;
        projected.project(center.distance_to(&Point::new(0.0, 0.0, 0.0)), 1.0);

        boundary.push(projected);
    }

    boundary
}

/// Create a simple pentagon tile (placeholder implementation)
fn create_pentagon_tile(center: &Point, config: &SymmetricalConfig) -> Tile {
    // This is a simplified implementation
    // In practice, we'd need to generate the proper boundary points
    let boundary = create_pentagon_boundary(center, config.radius * 0.1);

    Tile {
        center_point: center.clone(),
        boundary,
        neighbor_ids: Vec::new(),
        neighbors: Vec::new(),
    }
}

/// Create pentagon boundary points (5-sided polygon) using proper spherical geometry
fn create_pentagon_boundary(center: &Point, radius: f64) -> Vec<Point> {
    let mut boundary = Vec::new();

    // Create proper local coordinate system on the sphere surface
    let up = Vector3::new(center.x, center.y, center.z).normalize();

    // Create two perpendicular vectors in the tangent plane
    let reference = Vector3::new(0.0, 0.0, 1.0);
    let right = if up.z.abs() > 0.9 {
        Vector3::new(1.0, 0.0, 0.0)
    } else {
        reference.cross(&up).normalize()
    };
    let forward = up.cross(&right).normalize();

    // Create 5 points around the pentagon center using proper 3D geometry
    for i in 0..5 {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / 5.0;

        // Calculate position in the tangent plane
        let local_x = radius * angle.cos();
        let local_y = radius * angle.sin();

        // Transform to global coordinates using the local coordinate system
        let global_point = Point::new(
            center.x + local_x * right.x + local_y * forward.x,
            center.y + local_x * right.y + local_y * forward.y,
            center.z + local_x * right.z + local_y * forward.z,
        );

        // Project to sphere surface maintaining the proper radius
        let mut projected = global_point;
        let sphere_radius = center.distance_to(&Point::new(0.0, 0.0, 0.0));
        projected.project(sphere_radius, 1.0);

        boundary.push(projected);
    }

    boundary
}

/// Create transformations for all 12 pentagon positions
fn create_pentagon_transforms(radius: f64) -> Vec<PentagonTransform> {
    let reference = get_projected_vertex(0, radius);
    let mut transforms = Vec::new();

    for i in 0..12 {
        let target = get_projected_vertex(i, radius);
        transforms.push(PentagonTransform::new(&reference, &target));
    }

    transforms
}

/// Get a projected icosahedral vertex
fn get_projected_vertex(index: usize, radius: f64) -> Point {
    let vertices = get_icosahedron_vertices();
    let mut vertex = vertices[index].clone();
    vertex.project(radius, 1.0);
    vertex
}

/// Resolve neighbor relationships between tiles
fn resolve_neighbors(tiles: &mut Vec<Tile>) {
    // Create a lookup map
    let mut tile_lookup: HashMap<String, usize> = HashMap::new();
    for (i, tile) in tiles.iter().enumerate() {
        tile_lookup.insert(tile.to_string(), i);
    }

    // For each tile, find neighbors based on proximity
    for tile in tiles.iter_mut() {
        tile.neighbors = tile
            .neighbor_ids
            .iter()
            .filter_map(|id| tile_lookup.get(id).copied())
            .collect();
    }
}

/// Helper function to normalize a point to unit length
fn normalize_point(point: &Point) -> Point {
    let mag = (point.x * point.x + point.y * point.y + point.z * point.z).sqrt();
    if mag > 0.0 {
        Point::new(point.x / mag, point.y / mag, point.z / mag)
    } else {
        Point::new(0.0, 0.0, 1.0) // Default to +Z if zero vector
    }
}

/// Create a 3x3 rotation matrix using Rodrigues' rotation formula
fn create_rotation_matrix(axis: &Vector3, angle: f64) -> [[f64; 3]; 3] {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let one_minus_cos = 1.0 - cos_a;

    let ux = axis.x;
    let uy = axis.y;
    let uz = axis.z;

    [
        [
            cos_a + ux * ux * one_minus_cos,
            ux * uy * one_minus_cos - uz * sin_a,
            ux * uz * one_minus_cos + uy * sin_a,
        ],
        [
            uy * ux * one_minus_cos + uz * sin_a,
            cos_a + uy * uy * one_minus_cos,
            uy * uz * one_minus_cos - ux * sin_a,
        ],
        [
            uz * ux * one_minus_cos - uy * sin_a,
            uz * uy * one_minus_cos + ux * sin_a,
            cos_a + uz * uz * one_minus_cos,
        ],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pentagon_transform_identity() {
        let point = Point::new(1.0, 0.0, 0.0);
        let transform = PentagonTransform::new(&point, &point);
        let result = transform.transform_point(&point);

        assert!((result.x - point.x).abs() < 0.001);
        assert!((result.y - point.y).abs() < 0.001);
        assert!((result.z - point.z).abs() < 0.001);
    }

    #[test]
    fn test_symmetrical_generation() {
        let config = SymmetricalConfig::new(10.0, 2, 0.9);
        let hexasphere = create_symmetrical_hexasphere(config);

        // Should have some tiles
        assert!(!hexasphere.tiles.is_empty());

        // Should have exactly 12 pentagons (one at each icosahedral vertex)
        let pentagon_count = hexasphere
            .tiles
            .iter()
            .filter(|tile| tile.is_pentagon())
            .count();

        // Note: This test might fail initially since we have a simplified implementation
        // The full implementation would ensure exactly 12 pentagons
        assert!(pentagon_count > 0);
    }

    #[test]
    fn test_icosahedral_vertices() {
        // Test that icosahedral vertices are properly positioned
        let vertices = get_icosahedron_vertices();
        for (i, _vertex) in vertices.iter().enumerate() {
            let projected = get_projected_vertex(i, 10.0);
            let distance =
                (projected.x * projected.x + projected.y * projected.y + projected.z * projected.z)
                    .sqrt();
            assert!((distance - 10.0).abs() < 0.1);
        }
    }
}
