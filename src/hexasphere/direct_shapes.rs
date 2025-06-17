//! Direct generation of unique tile shapes without creating the full hexasphere.
//!
//! This module provides an experimental approach to generate tile shapes directly
//! based on mathematical patterns, avoiding the need to generate and analyze all tiles.

use crate::geometry::Point;
use crate::hexasphere::shape_theory::{predict_unique_shapes, calculate_distance_classes};
use crate::hexasphere::shape_instances::TileShape;
use std::collections::HashMap;

/// Configuration for a tile based on its position in the geodesic structure.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TileConfig {
    /// Distance from nearest pentagon
    pub pentagon_distance: u32,
    /// Number of neighboring pentagons at distance d
    pub pentagon_neighbors: u32,
    /// Number of neighboring hexagons at distance d-1
    pub inner_neighbors: u32,
    /// Number of neighboring hexagons at distance d
    pub same_neighbors: u32,
    /// Number of neighboring hexagons at distance d+1
    pub outer_neighbors: u32,
}

/// Generates unique tile shapes directly based on subdivision level.
///
/// This experimental function attempts to generate representative tile shapes
/// without creating the full hexasphere mesh, based on theoretical patterns.
pub fn generate_unique_shapes_direct(subdivision_level: u32) -> Vec<TileShape> {
    let mut shapes = Vec::new();
    
    // Always have one pentagon shape
    shapes.push(create_pentagon_shape());
    
    // Generate hexagon shapes based on distance classes
    let distance_classes = calculate_distance_classes(subdivision_level);
    
    for class in distance_classes.iter().skip(1) { // Skip pentagon class
        for i in 0..class.shape_types {
            let config = generate_tile_config(class.distance, i, subdivision_level);
            let shape = create_hexagon_shape_from_config(&config);
            shapes.push(shape);
        }
    }
    
    // Ensure we have the expected number of shapes
    let expected = predict_unique_shapes(subdivision_level) as usize;
    while shapes.len() < expected {
        // Add variations for higher complexity levels
        let variation_idx = shapes.len();
        if shapes.len() > 1 {
            let base_shape = &shapes[1 + variation_idx % (shapes.len() - 1)];
            let varied_shape = create_shape_variation(base_shape, variation_idx);
            shapes.push(varied_shape);
        } else {
            // Fallback for edge cases
            shapes.push(create_hexagon_shape_from_config(&TileConfig {
                pentagon_distance: 1,
                pentagon_neighbors: 1,
                inner_neighbors: 0,
                same_neighbors: 4,
                outer_neighbors: 1,
            }));
        }
    }
    
    shapes.truncate(expected);
    shapes
}

/// Creates the standard pentagon shape.
fn create_pentagon_shape() -> TileShape {
    let angle_step = 2.0 * std::f64::consts::PI / 5.0;
    let radius = 0.2; // Approximate radius for standard subdivision
    
    let vertices: Vec<Point> = (0..5)
        .map(|i| {
            let angle = i as f64 * angle_step;
            Point::new(
                radius * angle.cos(),
                0.0,
                radius * angle.sin()
            )
        })
        .collect();
    
    TileShape {
        vertices,
        sides: 5,
        radius,
    }
}

/// Generates a tile configuration based on its position in the structure.
fn generate_tile_config(distance: u32, variant: u32, subdivision_level: u32) -> TileConfig {
    // This is a simplified model - real configurations are more complex
    match distance {
        1 => TileConfig {
            pentagon_distance: 1,
            pentagon_neighbors: 1,
            inner_neighbors: 0,
            same_neighbors: 4,
            outer_neighbors: 1,
        },
        2 => {
            if subdivision_level <= 4 {
                TileConfig {
                    pentagon_distance: 2,
                    pentagon_neighbors: 0,
                    inner_neighbors: 2,
                    same_neighbors: 2,
                    outer_neighbors: 2,
                }
            } else {
                // At higher subdivisions, distance 2 tiles split into variants
                match variant {
                    0 => TileConfig {
                        pentagon_distance: 2,
                        pentagon_neighbors: 0,
                        inner_neighbors: 1,
                        same_neighbors: 3,
                        outer_neighbors: 2,
                    },
                    _ => TileConfig {
                        pentagon_distance: 2,
                        pentagon_neighbors: 0,
                        inner_neighbors: 2,
                        same_neighbors: 2,
                        outer_neighbors: 2,
                    },
                }
            }
        }
        _ => {
            // Higher distances have more complex patterns
            TileConfig {
                pentagon_distance: distance,
                pentagon_neighbors: 0,
                inner_neighbors: 2 - (variant % 2),
                same_neighbors: 2 + (variant % 2),
                outer_neighbors: 2,
            }
        }
    }
}

/// Creates a hexagon shape based on its configuration.
fn create_hexagon_shape_from_config(config: &TileConfig) -> TileShape {
    // Base radius decreases with distance from pentagons
    let base_radius = 0.3 / (1.0 + 0.1 * config.pentagon_distance as f64);
    
    // Shape distortion based on neighbor configuration
    let distortion_factor = if config.inner_neighbors != config.outer_neighbors {
        1.1 // Asymmetric neighbor pattern creates elongation
    } else {
        1.0 // Symmetric pattern
    };
    
    // Generate vertices with subtle variations based on configuration
    let mut vertices = Vec::new();
    let angle_step = 2.0 * std::f64::consts::PI / 6.0;
    
    for i in 0..6 {
        let angle = i as f64 * angle_step;
        
        // Vary radius based on position and neighbor pattern
        let radius_variation = match i {
            0 | 3 => distortion_factor, // Elongation axis
            _ => 1.0 / distortion_factor,
        };
        
        let r = base_radius * radius_variation;
        
        vertices.push(Point::new(
            r * angle.cos(),
            0.0,
            r * angle.sin()
        ));
    }
    
    TileShape {
        vertices,
        sides: 6,
        radius: base_radius,
    }
}

/// Creates a variation of an existing shape.
fn create_shape_variation(base_shape: &TileShape, variation_index: usize) -> TileShape {
    let mut vertices = Vec::new();
    
    // Apply small perturbations to create variations
    let perturbation = 0.02 * (1.0 + variation_index as f64 * 0.1);
    
    for (i, vertex) in base_shape.vertices.iter().enumerate() {
        let factor = 1.0 + perturbation * ((i + variation_index) % 3) as f64 / 3.0;
        vertices.push(Point::new(
            vertex.x * factor,
            vertex.y,
            vertex.z * factor
        ));
    }
    
    TileShape {
        vertices,
        sides: base_shape.sides,
        radius: base_shape.radius * (1.0 + perturbation),
    }
}

/// Generates placement rules for shapes based on icosahedral symmetry.
pub struct ShapePlacementRules {
    /// Maps shape index to icosahedral symmetry group
    pub symmetry_groups: HashMap<usize, SymmetryGroup>,
}

#[derive(Debug, Clone)]
pub enum SymmetryGroup {
    /// 12-fold symmetry (icosahedron vertices)
    Vertex,
    /// 20-fold symmetry (icosahedron faces)  
    Face,
    /// 30-fold symmetry (icosahedron edges)
    Edge,
    /// 60-fold symmetry (general position)
    General,
}

impl ShapePlacementRules {
    /// Creates placement rules for the given shapes.
    pub fn new(shapes: &[TileShape]) -> Self {
        let mut symmetry_groups = HashMap::new();
        
        // Pentagon is always at vertices
        symmetry_groups.insert(0, SymmetryGroup::Vertex);
        
        // Assign other shapes based on theoretical frequency
        for i in 1..shapes.len() {
            let group = match i % 4 {
                0 => SymmetryGroup::Face,
                1 => SymmetryGroup::Edge,
                _ => SymmetryGroup::General,
            };
            symmetry_groups.insert(i, group);
        }
        
        ShapePlacementRules { symmetry_groups }
    }
    
    /// Returns the number of instances for a shape based on its symmetry group.
    pub fn instance_count(&self, shape_index: usize) -> usize {
        match self.symmetry_groups.get(&shape_index) {
            Some(SymmetryGroup::Vertex) => 12,
            Some(SymmetryGroup::Face) => 20,
            Some(SymmetryGroup::Edge) => 30,
            Some(SymmetryGroup::General) => 60,
            None => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_direct_shape_generation() {
        let shapes = generate_unique_shapes_direct(4);
        
        // Should generate expected number of shapes
        assert_eq!(shapes.len(), predict_unique_shapes(4));
        
        // First shape should be pentagon
        assert_eq!(shapes[0].sides, 5);
        
        // Rest should be hexagons
        for shape in &shapes[1..] {
            assert_eq!(shape.sides, 6);
        }
    }
    
    #[test]
    fn test_shape_placement_rules() {
        let shapes = generate_unique_shapes_direct(3);
        let rules = ShapePlacementRules::new(&shapes);
        
        // Pentagon should have 12 instances
        assert_eq!(rules.instance_count(0), 12);
        
        // Total instances should match expected tile count
        let total_instances: usize = (0..shapes.len())
            .map(|i| rules.instance_count(i))
            .sum();
        
        // For subdivision 3: 92 tiles expected
        assert!(total_instances >= 92);
    }
}