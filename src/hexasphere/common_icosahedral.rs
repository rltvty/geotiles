//! Common icosahedral topology definitions shared across symmetrical implementations.

use crate::geometry::Point;
use crate::tile::core::Tile;

/// Icosahedral edges (30 total)
pub const ICOSAHEDRAL_EDGES: [(usize, usize); 30] = [
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
pub const ICOSAHEDRAL_FACES: [(usize, usize, usize); 20] = [
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
pub fn get_icosahedral_vertices() -> [Point; 12] {
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

/// Create a pentagon tile with proper boundary
pub fn create_pentagon_tile(center: &Point, hex_size: f64) -> Tile {
    let boundary = create_regular_polygon_boundary(center, 5, hex_size * 0.8);
    
    Tile {
        center_point: center.clone(),
        boundary,
        neighbor_ids: Vec::new(),
        neighbors: Vec::new(),
    }
}

/// Create regular polygon boundary on sphere surface
fn create_regular_polygon_boundary(center: &Point, sides: usize, radius: f64) -> Vec<Point> {
    use crate::geometry::Vector3;
    
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

/// Resolve neighbor relationships (simplified)
pub fn resolve_neighbors(_tiles: &mut Vec<Tile>) {
    // TODO: Implement neighbor resolution based on proximity
}