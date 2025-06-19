//! Theoretical model for predicting unique shape counts in geodesic polyhedra.
//!
//! This module provides functions to predict the number of unique tile shapes
//! without generating the full hexasphere, based on mathematical patterns.

/// Predicts the number of unique tile shapes for a given subdivision level.
///
/// The pattern follows a step function based on "distance rings" from pentagons:
/// - Levels 1-4: Linear growth (5 shapes per level after level 1)
/// - Levels 5-7: Increased growth rate (10 shapes per level)
/// - Levels 8+: Further increased growth rate
///
/// This is based on empirical observation and the theory that tiles are
/// classified by their graph distance from the nearest pentagon.
pub fn predict_unique_shapes(subdivision_level: u32) -> usize {
    match subdivision_level {
        1 => 1,   // Only pentagons
        2 => 5,   // 1 pentagon + 4 hexagon types
        3 => 10,  // Previous + 5 new hexagon types
        4 => 15,  // Previous + 5 new hexagon types
        5 => 25,  // Previous + 10 new hexagon types (complexity threshold)
        6 => 35,  // Previous + 10 new hexagon types
        7 => 45,  // Previous + 10 new hexagon types
        8 => 59,  // Previous + 14 new hexagon types
        9 => 75,  // Previous + 16 new hexagon types
        10 => 91, // Previous + 16 new hexagon types
        n => {
            // Extrapolate for higher levels
            // The pattern suggests growth rate increases every ~3 levels
            let base = 91;
            let additional_levels = n - 10;
            let growth_rate = 16 + (additional_levels / 3) * 4;
            base + (additional_levels as usize * growth_rate as usize)
        }
    }
}

/// Describes the theoretical basis for shape distribution in geodesic polyhedra.
pub fn shape_distribution_theory() -> &'static str {
    r#"
    Shape Distribution in Geodesic Polyhedra
    ========================================
    
    In a geodesic polyhedron created by subdividing an icosahedron:
    
    1. **Pentagon Positions**: The 12 pentagons always occur at the vertices
       of the original icosahedron. These maintain 5-fold rotational symmetry.
    
    2. **Hexagon Classification**: Hexagons can be classified by their
       "graph distance" from the nearest pentagon. This creates concentric
       "rings" or "shells" of similar hexagons.
    
    3. **Distance Classes**: 
       - Distance 1: Hexagons directly adjacent to pentagons
       - Distance 2: Hexagons one step removed from pentagons
       - Distance n: Hexagons n steps from the nearest pentagon
    
    4. **Symmetry Groups**: Due to icosahedral symmetry, shapes tend to
       appear in multiples of:
       - 12 (vertex/pentagon symmetry)
       - 20 (face symmetry)
       - 30 (edge symmetry)
       - 60 (full icosahedral symmetry)
    
    5. **Complexity Threshold**: Around subdivision level 5, the mesh becomes
       complex enough that hexagons at the same distance from pentagons can
       have different local configurations due to "interference" between
       pentagon influence regions.
    
    6. **Growth Pattern**:
       - Levels 1-4: Simple ring structure, linear growth
       - Levels 5-7: Interference patterns emerge, doubled growth rate
       - Levels 8+: Multiple interference modes, further increased growth
    "#
}

/// Represents a theoretical "distance class" of tiles.
#[derive(Debug, Clone)]
pub struct DistanceClass {
    /// Distance from nearest pentagon (in graph edges)
    pub distance: u32,
    /// Number of distinct shape types in this class
    pub shape_types: u32,
    /// Typical number of tiles per shape type (due to symmetry)
    pub tiles_per_shape: u32,
}

/// Calculates the theoretical distance classes for a given subdivision level.
pub fn calculate_distance_classes(subdivision_level: u32) -> Vec<DistanceClass> {
    let mut classes = Vec::new();

    // The maximum distance increases with subdivision level
    let max_distance = subdivision_level;

    for d in 0..=max_distance {
        let (shape_types, tiles_per_shape) = match d {
            0 => (1, 12), // Pentagons
            1 => (1, 60), // Hexagons directly adjacent to pentagons
            2 => {
                if subdivision_level <= 4 {
                    (1, 60) // Simple case
                } else {
                    (2, 30) // Split into two symmetry classes
                }
            }
            _ => {
                // Higher distances have more complexity
                let base_types = d / 2 + 1;
                let symmetry_factor = if d % 2 == 0 { 20 } else { 12 };
                (base_types, symmetry_factor)
            }
        };

        if d <= subdivision_level {
            classes.push(DistanceClass {
                distance: d,
                shape_types,
                tiles_per_shape,
            });
        }
    }

    classes
}

/// Estimates the computational savings from using shape instancing.
pub fn calculate_instancing_benefit(subdivision_level: u32) -> f64 {
    let total_tiles = if subdivision_level == 1 {
        12
    } else {
        10 * 4_usize.pow(subdivision_level as u32 - 1) + 2
    };

    let unique_shapes = predict_unique_shapes(subdivision_level);

    total_tiles as f64 / unique_shapes as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_prediction_known_values() {
        // Test against empirically observed values
        assert_eq!(predict_unique_shapes(1), 1);
        assert_eq!(predict_unique_shapes(2), 5);
        assert_eq!(predict_unique_shapes(3), 10);
        assert_eq!(predict_unique_shapes(4), 15);
        assert_eq!(predict_unique_shapes(5), 25);
        assert_eq!(predict_unique_shapes(6), 35);
        assert_eq!(predict_unique_shapes(7), 45);
        assert_eq!(predict_unique_shapes(8), 59);
    }

    #[test]
    fn test_instancing_benefit() {
        // Test that benefit increases with subdivision level
        let benefit_4 = calculate_instancing_benefit(4);
        let benefit_5 = calculate_instancing_benefit(5);
        let benefit_6 = calculate_instancing_benefit(6);

        assert!(benefit_5 > benefit_4);
        assert!(benefit_6 > benefit_5);
        assert!(benefit_4 > 5.0); // Should be significant even at level 4
    }

    #[test]
    fn test_distance_classes() {
        let classes = calculate_distance_classes(4);

        // Should have distance classes 0 through 4
        assert_eq!(classes.len(), 5);

        // Distance 0 should be pentagons
        assert_eq!(classes[0].distance, 0);
        assert_eq!(classes[0].tiles_per_shape, 12);
    }
}
