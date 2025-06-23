//! Proper symmetrical hexasphere generation with real 3D transformations.
//!
//! This implementation generates actual subdivision patterns and replicates them
//! using proper icosahedral group transformations.

use crate::geometry::{Point, Face, Vector3};
use crate::tile::core::Tile;
use crate::utils::{subdivide_face, find_projected_point, sort_faces_around_point};
use std::collections::HashMap;
use super::common_icosahedral::*;

/// Configuration for proper symmetrical generation
#[derive(Debug, Clone)]
pub struct ProperSymmetricalConfig {
    pub radius: f64,
    pub subdivisions: usize,
    pub hex_size: f64,
}

impl ProperSymmetricalConfig {
    pub fn new(radius: f64, subdivisions: usize, hex_size: f64) -> Self {
        if subdivisions == 0 {
            panic!("Subdivisions must be at least 1 for proper symmetrical generation");
        }
        
        Self {
            radius,
            subdivisions,
            hex_size,
        }
    }
}

/// Generate hexasphere using proper subdivision patterns and transformations
pub fn create_proper_symmetrical_hexasphere(
    config: ProperSymmetricalConfig,
) -> crate::hexasphere::core::Hexasphere {
    let vertices = get_icosahedral_vertices();
    let mut all_tiles = Vec::new();
    
    // Step 1: Generate the 12 pentagon tiles at icosahedral vertices
    for vertex in &vertices {
        let mut projected_vertex = vertex.clone();
        projected_vertex.project(config.radius, 1.0);
        let pentagon_tile = create_pentagon_tile(&projected_vertex, config.hex_size);
        all_tiles.push(pentagon_tile);
    }
    
    // Step 2: Generate edge patterns using real subdivision
    let edge_tiles = generate_edge_tiles_with_proper_transforms(&config, &vertices);
    all_tiles.extend(edge_tiles);
    
    // Step 3: Generate face patterns using real subdivision  
    let face_tiles = generate_face_tiles_with_proper_transforms(&config, &vertices);
    all_tiles.extend(face_tiles);
    
    // Step 4: Deduplicate tiles at boundaries
    let deduplicated_tiles = deduplicate_boundary_tiles(all_tiles);
    
    // Step 5: Resolve neighbor relationships
    let mut final_tiles = deduplicated_tiles;
    resolve_neighbors(&mut final_tiles);
    
    crate::hexasphere::core::Hexasphere {
        radius: config.radius,
        tiles: final_tiles,
    }
}

/// Generate edge tiles using real subdivision and proper transformations
fn generate_edge_tiles_with_proper_transforms(
    config: &ProperSymmetricalConfig,
    vertices: &[Point; 12]
) -> Vec<Tile> {
    if config.subdivisions <= 1 {
        return Vec::new(); // No edge hexagons for subdivision 1
    }
    
    // Step 1: Generate real subdivision pattern for the reference edge (0 -> 1)
    let edge_template = generate_real_edge_subdivision(&vertices[0], &vertices[1], config);
    
    // Step 2: Apply proper transformations to all 30 edges
    let mut all_edge_tiles = Vec::new();
    
    for &(v1_idx, v2_idx) in &ICOSAHEDRAL_EDGES {
        let target_v1 = &vertices[v1_idx];
        let target_v2 = &vertices[v2_idx];
        
        // Calculate proper rotation matrix from reference edge to target edge
        let transform = calculate_proper_edge_transform(
            &vertices[0], &vertices[1],  // Reference edge
            target_v1, target_v2,       // Target edge
            config.radius
        );
        
        // Apply transformation to each tile in the template
        for template_tile in &edge_template {
            let transformed_tile = apply_proper_transform_to_tile(template_tile, &transform);
            all_edge_tiles.push(transformed_tile);
        }
    }
    
    all_edge_tiles
}

/// Generate face tiles using real subdivision and proper transformations
fn generate_face_tiles_with_proper_transforms(
    config: &ProperSymmetricalConfig,
    vertices: &[Point; 12]
) -> Vec<Tile> {
    if config.subdivisions <= 2 {
        return Vec::new(); // No face hexagons for subdivision 2 or less
    }
    
    // Step 1: Generate real subdivision pattern for the reference face (0, 1, 4)
    let face_template = generate_real_face_subdivision(&vertices[0], &vertices[1], &vertices[4], config);
    
    // Step 2: Apply proper transformations to all 20 faces
    let mut all_face_tiles = Vec::new();
    
    for &(v1_idx, v2_idx, v3_idx) in &ICOSAHEDRAL_FACES {
        let target_v1 = &vertices[v1_idx];
        let target_v2 = &vertices[v2_idx];
        let target_v3 = &vertices[v3_idx];
        
        // Calculate proper rotation matrix from reference triangle to target triangle
        let transform = calculate_proper_face_transform(
            &vertices[0], &vertices[1], &vertices[4],  // Reference triangle
            target_v1, target_v2, target_v3,          // Target triangle
            config.radius
        );
        
        // Apply transformation to each tile in the template
        for template_tile in &face_template {
            let transformed_tile = apply_proper_transform_to_tile(template_tile, &transform);
            all_face_tiles.push(transformed_tile);
        }
    }
    
    all_face_tiles
}

/// Generate real subdivision pattern for an edge using the actual subdivision algorithm
fn generate_real_edge_subdivision(v1: &Point, v2: &Point, config: &ProperSymmetricalConfig) -> Vec<Tile> {
    // Create a minimal icosahedral face that includes this edge
    // Use the actual third vertex from the icosahedron for a real face
    let vertices = get_icosahedral_vertices();
    let v3 = &vertices[4]; // Use vertex 4 as the third point for face (0,1,4)
    
    // Generate the complete face using real subdivision
    let face_tiles = generate_complete_face_subdivision(v1, v2, v3, config);
    
    // Extract only the tiles that lie along the edge v1-v2
    extract_edge_tiles_from_face(&face_tiles, v1, v2)
}

/// Generate real subdivision pattern for a face using the actual subdivision algorithm
fn generate_real_face_subdivision(v1: &Point, v2: &Point, v3: &Point, config: &ProperSymmetricalConfig) -> Vec<Tile> {
    // Generate the complete face using real subdivision
    let face_tiles = generate_complete_face_subdivision(v1, v2, v3, config);
    
    // Extract only the interior tiles (not on edges)
    extract_interior_tiles_from_face(&face_tiles, v1, v2, v3)
}

/// Generate complete face subdivision using the actual algorithm from core.rs
fn generate_complete_face_subdivision(v1: &Point, v2: &Point, v3: &Point, config: &ProperSymmetricalConfig) -> Vec<Tile> {
    // Use the same algorithm as in core.rs but for a single face
    
    let mut points: HashMap<Point, Point> = HashMap::new();
    
    // Add triangle vertices
    points.insert(v1.clone(), v1.clone());
    points.insert(v2.clone(), v2.clone());
    points.insert(v3.clone(), v3.clone());
    
    // Create the initial face
    let face = Face::new(0, v1.clone(), v2.clone(), v3.clone());
    
    // Subdivide using the same algorithm as core.rs
    let mut face_id = 1;
    let subdivided_faces = subdivide_face(face, config.subdivisions, &mut points, &mut face_id);
    
    // Project all points to sphere (same as core.rs)
    let mut projected_points: HashMap<Point, Point> = HashMap::new();
    for point in points.into_values() {
        let mut projected = point.clone();
        projected.project(config.radius, 1.0);
        projected_points.insert(projected.clone(), projected);
    }
    
    // Update faces with projected vertices (same as core.rs)
    let mut updated_faces = subdivided_faces;
    for face in &mut updated_faces {
        for i in 0..3 {
            if let Some(projected_point) = find_projected_point(&face.points[i], &projected_points) {
                face.points[i] = projected_point;
            }
        }
        face.clear_centroid_cache();
    }
    
    // Convert faces to tiles using the same dual algorithm as core.rs
    convert_faces_to_tiles(&updated_faces, config)
}

/// Convert subdivided faces to tiles using the same dual algorithm as core.rs
fn convert_faces_to_tiles(faces: &[Face], config: &ProperSymmetricalConfig) -> Vec<Tile> {
    // Group faces by their vertices (same as core.rs)
    let mut point_to_faces: HashMap<Point, Vec<usize>> = HashMap::new();
    for (face_idx, face) in faces.iter().enumerate() {
        for point in &face.points {
            point_to_faces
                .entry(point.clone())
                .or_default()
                .push(face_idx);
        }
    }
    
    // Create tiles (same as core.rs)
    let mut tiles = Vec::new();
    
    for (point, face_indices) in point_to_faces {
        let mut point_faces: Vec<Face> = face_indices
            .into_iter()
            .map(|idx| faces[idx].clone())
            .collect();
        
        // Sort faces around the point (same as core.rs)
        sort_faces_around_point(&mut point_faces, &point);
        
        let tile = Tile::new(point, &mut point_faces, config.hex_size);
        tiles.push(tile);
    }
    
    tiles
}

/// Extract tiles that lie along the specified edge
fn extract_edge_tiles_from_face(face_tiles: &[Tile], v1: &Point, v2: &Point) -> Vec<Tile> {
    let mut edge_tiles = Vec::new();
    
    // Calculate edge vector and length
    let edge_vec = Vector3::new(v2.x - v1.x, v2.y - v1.y, v2.z - v1.z);
    let edge_length = edge_vec.magnitude();
    let edge_dir = edge_vec.normalize();
    
    for tile in face_tiles {
        // Check if tile center is close to the edge line
        let to_tile = Vector3::new(
            tile.center_point.x - v1.x,
            tile.center_point.y - v1.y,
            tile.center_point.z - v1.z
        );
        
        // Project onto edge direction
        let projection_length = to_tile.dot(&edge_dir);
        let projection = Vector3::new(
            edge_dir.x * projection_length,
            edge_dir.y * projection_length,
            edge_dir.z * projection_length
        );
        
        // Calculate distance from tile to edge line
        let distance_vec = Vector3::new(
            to_tile.x - projection.x,
            to_tile.y - projection.y,
            to_tile.z - projection.z
        );
        let distance_to_edge = distance_vec.magnitude();
        
        // If close to edge and between v1 and v2, include it
        let tolerance = edge_length * 0.2; // More generous tolerance
        let margin = edge_length * 0.1; // Margin beyond endpoints
        if distance_to_edge < tolerance && projection_length > -margin && projection_length < (edge_length + margin) {
            edge_tiles.push(tile.clone());
        }
    }
    
    edge_tiles
}

/// Extract tiles that lie in the interior of the triangle (not on edges)
fn extract_interior_tiles_from_face(face_tiles: &[Tile], v1: &Point, v2: &Point, v3: &Point) -> Vec<Tile> {
    let mut interior_tiles = Vec::new();
    
    for tile in face_tiles {
        // Check if tile is interior using barycentric coordinates
        let barycentric = calculate_barycentric_coordinates(&tile.center_point, v1, v2, v3);
        
        // If all coordinates are positive and away from edges, it's interior  
        let min_coord = barycentric.0.min(barycentric.1).min(barycentric.2);
        if min_coord > 0.05 { // More generous interior threshold
            interior_tiles.push(tile.clone());
        }
    }
    
    interior_tiles
}

/// Calculate barycentric coordinates of a point relative to a triangle in 3D
fn calculate_barycentric_coordinates(p: &Point, v1: &Point, v2: &Point, v3: &Point) -> (f64, f64, f64) {
    // Calculate triangle edges
    let v0 = Vector3::new(v2.x - v1.x, v2.y - v1.y, v2.z - v1.z);
    let v1_vec = Vector3::new(v3.x - v1.x, v3.y - v1.y, v3.z - v1.z);
    let v2_vec = Vector3::new(p.x - v1.x, p.y - v1.y, p.z - v1.z);
    
    // Calculate dot products
    let dot00 = v0.dot(&v0);
    let dot01 = v0.dot(&v1_vec);
    let dot02 = v0.dot(&v2_vec);
    let dot11 = v1_vec.dot(&v1_vec);
    let dot12 = v1_vec.dot(&v2_vec);
    
    // Calculate barycentric coordinates
    let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;
    
    (1.0 - u - v, u, v)
}

/// Calculate proper rotation matrix to transform one edge to another
fn calculate_proper_edge_transform(
    ref_v1: &Point, ref_v2: &Point,
    target_v1: &Point, target_v2: &Point,
    radius: f64
) -> [[f64; 3]; 3] {
    // Project vertices to sphere surface
    let mut ref_v1_proj = ref_v1.clone();
    let mut ref_v2_proj = ref_v2.clone();
    let mut target_v1_proj = target_v1.clone();
    let mut target_v2_proj = target_v2.clone();
    
    ref_v1_proj.project(radius, 1.0);
    ref_v2_proj.project(radius, 1.0);
    target_v1_proj.project(radius, 1.0);
    target_v2_proj.project(radius, 1.0);
    
    // Calculate edge vectors
    let ref_edge = Vector3::new(
        ref_v2_proj.x - ref_v1_proj.x,
        ref_v2_proj.y - ref_v1_proj.y,
        ref_v2_proj.z - ref_v1_proj.z
    ).normalize();
    
    let target_edge = Vector3::new(
        target_v2_proj.x - target_v1_proj.x,
        target_v2_proj.y - target_v1_proj.y,
        target_v2_proj.z - target_v1_proj.z
    ).normalize();
    
    // Calculate rotation axis (cross product)
    let rotation_axis = ref_edge.cross(&target_edge).normalize();
    
    // Calculate rotation angle
    let cos_angle = ref_edge.dot(&target_edge).clamp(-1.0, 1.0);
    let angle = cos_angle.acos();
    
    // Create rotation matrix using Rodrigues' rotation formula
    if angle.abs() < 1e-10 {
        // No rotation needed
        [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
    } else {
        rodrigues_rotation_matrix(&rotation_axis, angle)
    }
}

/// Calculate proper rotation matrix to transform one triangle to another
fn calculate_proper_face_transform(
    ref_v1: &Point, ref_v2: &Point, ref_v3: &Point,
    target_v1: &Point, target_v2: &Point, target_v3: &Point,
    radius: f64
) -> [[f64; 3]; 3] {
    // Project vertices to sphere surface
    let mut ref_v1_proj = ref_v1.clone();
    let mut ref_v2_proj = ref_v2.clone();
    let mut ref_v3_proj = ref_v3.clone();
    let mut target_v1_proj = target_v1.clone();
    let mut target_v2_proj = target_v2.clone();
    let mut target_v3_proj = target_v3.clone();
    
    ref_v1_proj.project(radius, 1.0);
    ref_v2_proj.project(radius, 1.0);
    ref_v3_proj.project(radius, 1.0);
    target_v1_proj.project(radius, 1.0);
    target_v2_proj.project(radius, 1.0);
    target_v3_proj.project(radius, 1.0);
    
    // Calculate triangle normals
    let ref_normal = calculate_triangle_normal(&ref_v1_proj, &ref_v2_proj, &ref_v3_proj);
    let target_normal = calculate_triangle_normal(&target_v1_proj, &target_v2_proj, &target_v3_proj);
    
    // Calculate rotation axis (cross product)
    let rotation_axis = ref_normal.cross(&target_normal).normalize();
    
    // Calculate rotation angle
    let cos_angle = ref_normal.dot(&target_normal).clamp(-1.0, 1.0);
    let angle = cos_angle.acos();
    
    // Create rotation matrix using Rodrigues' rotation formula
    if angle.abs() < 1e-10 {
        // No rotation needed
        [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
    } else {
        rodrigues_rotation_matrix(&rotation_axis, angle)
    }
}

/// Calculate triangle normal vector
fn calculate_triangle_normal(v1: &Point, v2: &Point, v3: &Point) -> Vector3 {
    let edge1 = Vector3::new(v2.x - v1.x, v2.y - v1.y, v2.z - v1.z);
    let edge2 = Vector3::new(v3.x - v1.x, v3.y - v1.y, v3.z - v1.z);
    edge1.cross(&edge2).normalize()
}

/// Create rotation matrix using Rodrigues' rotation formula
fn rodrigues_rotation_matrix(axis: &Vector3, angle: f64) -> [[f64; 3]; 3] {
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

/// Apply proper transformation matrix to a tile
fn apply_proper_transform_to_tile(tile: &Tile, transform: &[[f64; 3]; 3]) -> Tile {
    let transform_point = |p: &Point| -> Point {
        let x = p.x * transform[0][0] + p.y * transform[0][1] + p.z * transform[0][2];
        let y = p.x * transform[1][0] + p.y * transform[1][1] + p.z * transform[1][2];
        let z = p.x * transform[2][0] + p.y * transform[2][1] + p.z * transform[2][2];
        Point::new(x, y, z)
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

/// Deduplicate tiles that appear at pattern boundaries
fn deduplicate_boundary_tiles(tiles: Vec<Tile>) -> Vec<Tile> {
    let mut unique_tiles: Vec<Tile> = Vec::new();
    
    for tile in tiles {
        let mut is_duplicate = false;
        
        // Check against existing tiles for proximity
        for existing_tile in &unique_tiles {
            let distance = tile.center_point.distance_to(&existing_tile.center_point);
            if distance < 0.1 { // Tiles are considered duplicates if centers are very close
                is_duplicate = true;
                break;
            }
        }
        
        if !is_duplicate {
            unique_tiles.push(tile);
        }
    }
    
    unique_tiles
}

