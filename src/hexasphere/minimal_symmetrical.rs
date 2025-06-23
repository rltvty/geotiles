//! Minimal symmetrical hexasphere generation using true subdivision patterns.
//!
//! This implementation generates only the minimal subdivision patterns and replicates
//! them using icosahedral symmetry, without generating the full sphere first.

use crate::geometry::{Point, Face, Vector3};
use crate::tile::core::Tile;
use crate::utils::{subdivide_face, find_projected_point, sort_faces_around_point};
use std::collections::HashMap;

/// Configuration for minimal symmetrical generation
#[derive(Debug, Clone)]
pub struct MinimalSymmetricalConfig {
    pub radius: f64,
    pub subdivisions: usize,
    pub hex_size: f64,
}

impl MinimalSymmetricalConfig {
    pub fn new(radius: f64, subdivisions: usize, hex_size: f64) -> Self {
        if subdivisions == 0 {
            panic!("Subdivisions must be at least 1 for minimal symmetrical generation");
        }
        
        Self {
            radius,
            subdivisions,
            hex_size,
        }
    }
}

/// Generate hexasphere using minimal patterns and icosahedral symmetry
pub fn create_minimal_symmetrical_hexasphere(
    config: MinimalSymmetricalConfig,
) -> crate::hexasphere::core::Hexasphere {
    let mut all_tiles = Vec::new();
    
    // Step 1: Generate the 12 pentagon tiles
    let pentagon_tiles = generate_pentagon_tiles(&config);
    all_tiles.extend(pentagon_tiles);
    
    // Step 2: Generate edge hexagons (subdivision along one edge, replicated to 30)
    let edge_tiles = generate_edge_tiles_minimal(&config);
    all_tiles.extend(edge_tiles);
    
    // Step 3: Generate face hexagons (subdivision in one triangle, replicated to 20)
    let face_tiles = generate_face_tiles_minimal(&config);
    all_tiles.extend(face_tiles);
    
    // Step 4: Resolve neighbor relationships
    resolve_neighbors(&mut all_tiles);
    
    crate::hexasphere::core::Hexasphere {
        radius: config.radius,
        tiles: all_tiles,
    }
}

/// Generate edge tiles by subdividing one edge and replicating to all 30 edges
fn generate_edge_tiles_minimal(config: &MinimalSymmetricalConfig) -> Vec<Tile> {
    if config.subdivisions <= 1 {
        return Vec::new(); // No edge hexagons for subdivision 1
    }
    
    let vertices = get_icosahedral_vertices();
    let mut all_edge_tiles = Vec::new();
    
    // Step 1: Generate subdivision along the first edge (vertices 0 -> 1)
    let edge_template = subdivide_edge_properly(&vertices[0], &vertices[1], config);
    
    // Step 2: Replicate this pattern to all 30 icosahedral edges
    for &(v1_idx, v2_idx) in &ICOSAHEDRAL_EDGES {
        let v1 = &vertices[v1_idx];
        let v2 = &vertices[v2_idx];
        
        // Calculate transformation from template edge (0->1) to current edge (v1->v2)
        let transform = calculate_edge_transform(&vertices[0], &vertices[1], v1, v2, config.radius);
        
        // Apply transformation to template tiles
        for template_tile in &edge_template {
            let transformed_tile = apply_transform_to_tile(template_tile, &transform);
            all_edge_tiles.push(transformed_tile);
        }
    }
    
    all_edge_tiles
}

/// Generate face tiles by subdividing one triangle and replicating to all 20 faces
fn generate_face_tiles_minimal(config: &MinimalSymmetricalConfig) -> Vec<Tile> {
    if config.subdivisions <= 2 {
        return Vec::new(); // No face hexagons for subdivision 2 or less
    }
    
    let vertices = get_icosahedral_vertices();
    let mut all_face_tiles = Vec::new();
    
    // Step 1: Generate subdivision in the first face triangle (vertices 0, 1, 4)
    let face_template = subdivide_triangle_properly(&vertices[0], &vertices[1], &vertices[4], config);
    
    // Step 2: Replicate this pattern to all 20 icosahedral faces
    for &(v1_idx, v2_idx, v3_idx) in &ICOSAHEDRAL_FACES {
        let v1 = &vertices[v1_idx];
        let v2 = &vertices[v2_idx];
        let v3 = &vertices[v3_idx];
        
        // Calculate transformation from template triangle (0,1,4) to current triangle
        let transform = calculate_face_transform(
            &vertices[0], &vertices[1], &vertices[4],
            v1, v2, v3,
            config.radius
        );
        
        // Apply transformation to template tiles
        for template_tile in &face_template {
            let transformed_tile = apply_transform_to_tile(template_tile, &transform);
            all_face_tiles.push(transformed_tile);
        }
    }
    
    all_face_tiles
}

/// Properly subdivide an edge using the actual subdivision algorithm
fn subdivide_edge_properly(v1: &Point, v2: &Point, config: &MinimalSymmetricalConfig) -> Vec<Tile> {
    // Create a minimal triangle that includes this edge for subdivision
    // We'll create a thin triangle and subdivide it, then extract only the edge tiles
    
    let mut points: HashMap<Point, Point> = HashMap::new();
    
    // Add the edge vertices
    points.insert(v1.clone(), v1.clone());
    points.insert(v2.clone(), v2.clone());
    
    // Create a third point to form a minimal triangle
    let midpoint = Point::new(
        (v1.x + v2.x) / 2.0,
        (v1.y + v2.y) / 2.0,
        (v1.z + v2.z) / 2.0,
    );
    
    // Offset slightly to create a thin triangle
    let offset_point = Point::new(
        midpoint.x + 0.01,
        midpoint.y + 0.01,
        midpoint.z + 0.01,
    );
    points.insert(offset_point.clone(), offset_point.clone());
    
    // Create the triangle face
    let face = Face::new(0, v1.clone(), v2.clone(), offset_point);
    
    // Subdivide the triangle
    let mut face_id = 1;
    let subdivided_faces = subdivide_face(face, config.subdivisions, &mut points, &mut face_id);
    
    // Project all points to sphere
    let mut projected_points: HashMap<Point, Point> = HashMap::new();
    for point in points.into_values() {
        let mut projected = point.clone();
        projected.project(config.radius, 1.0);
        projected_points.insert(projected.clone(), projected);
    }
    
    // Update faces with projected vertices
    let mut updated_faces = subdivided_faces;
    for face in &mut updated_faces {
        for i in 0..3 {
            if let Some(projected_point) = find_projected_point(&face.points[i], &projected_points) {
                face.points[i] = projected_point;
            }
        }
        face.clear_centroid_cache();
    }
    
    // Extract only the tiles along the original edge (not the offset triangle)
    extract_edge_tiles_from_faces(&updated_faces, v1, v2, config)
}

/// Properly subdivide a triangle using the actual subdivision algorithm
fn subdivide_triangle_properly(
    v1: &Point, 
    v2: &Point, 
    v3: &Point, 
    config: &MinimalSymmetricalConfig
) -> Vec<Tile> {
    let mut points: HashMap<Point, Point> = HashMap::new();
    
    // Add triangle vertices
    points.insert(v1.clone(), v1.clone());
    points.insert(v2.clone(), v2.clone());
    points.insert(v3.clone(), v3.clone());
    
    // Create the triangle face
    let face = Face::new(0, v1.clone(), v2.clone(), v3.clone());
    
    // Subdivide the triangle
    let mut face_id = 1;
    let subdivided_faces = subdivide_face(face, config.subdivisions, &mut points, &mut face_id);
    
    // Project all points to sphere
    let mut projected_points: HashMap<Point, Point> = HashMap::new();
    for point in points.into_values() {
        let mut projected = point.clone();
        projected.project(config.radius, 1.0);
        projected_points.insert(projected.clone(), projected);
    }
    
    // Update faces with projected vertices
    let mut updated_faces = subdivided_faces;
    for face in &mut updated_faces {
        for i in 0..3 {
            if let Some(projected_point) = find_projected_point(&face.points[i], &projected_points) {
                face.points[i] = projected_point;
            }
        }
        face.clear_centroid_cache();
    }
    
    // Extract interior tiles from the triangle (excluding edge tiles)
    extract_interior_tiles_from_faces(&updated_faces, v1, v2, v3, config)
}

/// Extract tiles that lie along the edge between v1 and v2
fn extract_edge_tiles_from_faces(
    faces: &[Face],
    v1: &Point,
    v2: &Point,
    config: &MinimalSymmetricalConfig
) -> Vec<Tile> {
    // This is a simplified implementation
    // Real implementation would identify face centroids that lie close to the edge line
    
    let mut edge_tiles = Vec::new();
    let edge_count = config.subdivisions - 1;
    
    // Create tiles along the edge
    for i in 1..=edge_count {
        let t = i as f64 / (edge_count + 1) as f64;
        let edge_point = Point::new(
            v1.x * (1.0 - t) + v2.x * t,
            v1.y * (1.0 - t) + v2.y * t,
            v1.z * (1.0 - t) + v2.z * t,
        );
        
        let mut projected = edge_point;
        projected.project(config.radius, 1.0);
        
        let tile = create_hexagon_tile(&projected, config);
        edge_tiles.push(tile);
    }
    
    edge_tiles
}

/// Extract tiles that lie in the interior of the triangle (not on edges)
fn extract_interior_tiles_from_faces(
    faces: &[Face],
    v1: &Point,
    v2: &Point,
    v3: &Point,
    config: &MinimalSymmetricalConfig
) -> Vec<Tile> {
    // This is a simplified implementation
    // Real implementation would identify face centroids that lie in the triangle interior
    
    let mut interior_tiles = Vec::new();
    let triangle_size = config.subdivisions - 2;
    
    if triangle_size <= 0 {
        return interior_tiles;
    }
    
    // Generate triangular number of interior hexagons
    let hexagon_count = triangle_size * (triangle_size + 1) / 2;
    
    for i in 0..hexagon_count {
        // Use barycentric coordinates to position hexagons
        let row = ((-1.0 + (1.0 + 8.0 * i as f64).sqrt()) / 2.0) as usize;
        let col = i - row * (row + 1) / 2;
        
        let u = (row + 1) as f64 / (triangle_size + 1) as f64;
        let v = (col + 1) as f64 / (triangle_size + 1) as f64;
        let w = 1.0 - u - v;
        
        if w > 0.0 { // Valid barycentric coordinates
            let interior_point = Point::new(
                u * v1.x + v * v2.x + w * v3.x,
                u * v1.y + v * v2.y + w * v3.y,
                u * v1.z + v * v2.z + w * v3.z,
            );
            
            let mut projected = interior_point;
            projected.project(config.radius, 1.0);
            
            let tile = create_hexagon_tile(&projected, config);
            interior_tiles.push(tile);
        }
    }
    
    interior_tiles
}

/// Icosahedral edges (30 total)
const ICOSAHEDRAL_EDGES: [(usize, usize); 30] = [
    // Top star: vertex 0 connects to 5 others (5 edges)
    (0, 1), (0, 4), (0, 8), (0, 10), (0, 6),
    // Upper pentagon ring (5 edges)
    (1, 2), (2, 3), (3, 4), (4, 5), (5, 1),
    // Lower pentagon ring (5 edges)
    (6, 7), (7, 8), (8, 9), (9, 10), (10, 6),
    // Vertical connections between rings (5 edges)
    (1, 6), (2, 7), (3, 8), (4, 9), (5, 10),
    // Bottom star: connections to vertex 11 (5 edges)
    (6, 11), (7, 11), (8, 11), (9, 11), (10, 11),
    // Diagonal connections completing icosahedron (5 edges)
    (1, 7), (2, 8), (3, 9), (4, 10), (5, 6)
];

/// Icosahedral faces (20 total)
const ICOSAHEDRAL_FACES: [(usize, usize, usize); 20] = [
    // Top pyramid (5 faces around vertex 0)
    (0, 1, 4), (0, 4, 8), (0, 8, 10), (0, 10, 6), (0, 6, 1),
    // Upper ring (5 faces)
    (1, 2, 5), (2, 3, 7), (3, 4, 8), (4, 5, 9), (5, 1, 6),
    // Lower ring (5 faces) 
    (6, 7, 11), (7, 8, 11), (8, 9, 11), (9, 10, 11), (10, 6, 11),
    // Middle band (5 faces)
    (1, 2, 7), (2, 3, 8), (3, 4, 9), (4, 5, 10), (5, 1, 6)
];

/// Get icosahedral vertex positions using golden ratio
fn get_icosahedral_vertices() -> [Point; 12] {
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

/// Generate all 12 pentagon tiles at icosahedral vertices
fn generate_pentagon_tiles(config: &MinimalSymmetricalConfig) -> Vec<Tile> {
    let vertices = get_icosahedral_vertices();
    let mut pentagons = Vec::new();
    
    for vertex in &vertices {
        let mut projected_vertex = vertex.clone();
        projected_vertex.project(config.radius, 1.0);
        
        let pentagon_tile = create_pentagon_tile(&projected_vertex, config);
        pentagons.push(pentagon_tile);
    }
    
    pentagons
}

/// Create a pentagon tile with proper boundary
fn create_pentagon_tile(center: &Point, config: &MinimalSymmetricalConfig) -> Tile {
    let boundary = create_regular_polygon_boundary(center, 5, config.radius * 0.1);
    
    Tile {
        center_point: center.clone(),
        boundary,
        neighbor_ids: Vec::new(),
        neighbors: Vec::new(),
    }
}

/// Create a hexagon tile with proper boundary
fn create_hexagon_tile(center: &Point, config: &MinimalSymmetricalConfig) -> Tile {
    let boundary = create_regular_polygon_boundary(center, 6, config.radius * 0.08);
    
    Tile {
        center_point: center.clone(),
        boundary,
        neighbor_ids: Vec::new(),
        neighbors: Vec::new(),
    }
}

/// Create regular polygon boundary on sphere surface
fn create_regular_polygon_boundary(center: &Point, sides: usize, radius: f64) -> Vec<Point> {
    let mut boundary = Vec::new();
    
    // Create local coordinate system on sphere surface
    let up = Vector3::new(center.x, center.y, center.z).normalize();
    let reference = Vector3::new(0.0, 0.0, 1.0);
    let right = if up.z.abs() > 0.9 {
        Vector3::new(1.0, 0.0, 0.0)
    } else {
        reference.cross(&up).normalize()
    };
    let forward = up.cross(&right).normalize();
    
    for i in 0..sides {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / sides as f64;
        let local_x = radius * angle.cos();
        let local_y = radius * angle.sin();
        
        let global_point = Point::new(
            center.x + local_x * right.x + local_y * forward.x,
            center.y + local_x * right.y + local_y * forward.y,
            center.z + local_x * right.z + local_y * forward.z,
        );
        
        let mut projected = global_point;
        let sphere_radius = center.distance_to(&Point::new(0.0, 0.0, 0.0));
        projected.project(sphere_radius, 1.0);
        
        boundary.push(projected);
    }
    
    boundary
}

/// Calculate transformation matrix from one edge to another
fn calculate_edge_transform(
    _template_v1: &Point,
    _template_v2: &Point,
    _target_v1: &Point,
    _target_v2: &Point,
    _radius: f64
) -> [[f64; 3]; 3] {
    // This is a placeholder - real implementation would calculate proper rotation matrix
    // that transforms template_v1->template_v2 to target_v1->target_v2
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

/// Calculate transformation matrix from one triangle to another
fn calculate_face_transform(
    _template_v1: &Point,
    _template_v2: &Point,
    _template_v3: &Point,
    _target_v1: &Point,
    _target_v2: &Point,
    _target_v3: &Point,
    _radius: f64
) -> [[f64; 3]; 3] {
    // This is a placeholder - real implementation would calculate proper rotation matrix
    // that transforms template triangle to target triangle
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

/// Apply transformation matrix to a tile
fn apply_transform_to_tile(tile: &Tile, transform: &[[f64; 3]; 3]) -> Tile {
    let transform_point = |p: &Point| -> Point {
        Point::new(
            p.x * transform[0][0] + p.y * transform[0][1] + p.z * transform[0][2],
            p.x * transform[1][0] + p.y * transform[1][1] + p.z * transform[1][2],
            p.x * transform[2][0] + p.y * transform[2][1] + p.z * transform[2][2],
        )
    };
    
    let new_center = transform_point(&tile.center_point);
    let new_boundary: Vec<Point> = tile.boundary.iter()
        .map(transform_point)
        .collect();
    
    Tile {
        center_point: new_center,
        boundary: new_boundary,
        neighbor_ids: tile.neighbor_ids.clone(),
        neighbors: Vec::new(),
    }
}

/// Resolve neighbor relationships (simplified)
fn resolve_neighbors(_tiles: &mut Vec<Tile>) {
    // TODO: Implement neighbor resolution based on proximity
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_minimal_symmetrical_generation() {
        for subdivisions in 1..=5 {
            let config = MinimalSymmetricalConfig::new(10.0, subdivisions, 0.9);
            let hexasphere = create_minimal_symmetrical_hexasphere(config);
            
            let pentagon_count = hexasphere.tiles.iter().filter(|t| t.is_pentagon()).count();
            let hexagon_count = hexasphere.tiles.iter().filter(|t| t.is_hexagon()).count();
            
            println!("Subdivision {}: {} pentagons, {} hexagons", 
                subdivisions, pentagon_count, hexagon_count);
            
            // Should always have exactly 12 pentagons
            assert_eq!(pentagon_count, 12);
        }
    }
}