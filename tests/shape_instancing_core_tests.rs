use geotiles::{Hexasphere, ShapeAnalyzer, TileShape};
use std::collections::HashSet;

/// Test that shape instancing produces the same tile geometry as the original approach
#[test]
fn test_shape_instances_preserve_geometry() {
    for subdivision in 2..=6 {
        let hexasphere = Hexasphere::new(1.0, subdivision, 0.95);
        let shape_data = hexasphere.get_shape_instances();

        // Verify we have the same number of instances as original tiles
        assert_eq!(
            shape_data.instances.len(),
            hexasphere.tiles.len(),
            "Instance count should match tile count at subdivision {}",
            subdivision
        );

        // Verify that reconstructed tiles match original tiles
        for instance in shape_data.instances.iter() {
            let original_tile = &hexasphere.tiles[instance.tile_index];
            let shape = &shape_data.shapes[instance.shape_index];

            // Check that the shapes have the correct number of sides
            assert_eq!(
                shape.sides,
                original_tile.boundary.len(),
                "Shape sides should match original tile boundary length"
            );

            // Check that the center matches
            let center_distance = ((original_tile.center_point.x - instance.center.x).powi(2)
                + (original_tile.center_point.y - instance.center.y).powi(2)
                + (original_tile.center_point.z - instance.center.z).powi(2))
            .sqrt();
            assert!(
                center_distance < 0.001,
                "Center points should match within tolerance"
            );
        }
    }
}

/// Test that unique shape count follows the discovered pattern
#[test]
fn test_unique_shape_count_pattern() {
    let expected_counts = [
        (1, 1),  // Pentagon only
        (2, 5),  // 1 pentagon + 4 hexagon types
        (3, 10), // Linear growth phase
        (4, 15), // Linear growth phase
        (5, 25), // Growth rate doubles
        (6, 35), // Continued higher growth
        (7, 45), // Continued higher growth
        (8, 59), // Further increase
    ];

    for (subdivision, expected) in expected_counts {
        let hexasphere = Hexasphere::new(1.0, subdivision, 1.0);
        let shape_data = hexasphere.get_shape_instances();

        assert_eq!(
            shape_data.shapes.len(),
            expected,
            "Subdivision {} should have {} unique shapes",
            subdivision,
            expected
        );
    }
}

/// Test that all tiles are properly represented in shape instances
#[test]
fn test_complete_tile_coverage() {
    for subdivision in 2..=5 {
        let hexasphere = Hexasphere::new(1.0, subdivision, 1.0);
        let shape_data = hexasphere.get_shape_instances();

        // Collect all tile indices from instances
        let mut covered_tiles: HashSet<usize> = HashSet::new();
        for instance in &shape_data.instances {
            covered_tiles.insert(instance.tile_index);
        }

        // Verify all tiles are covered
        for i in 0..hexasphere.tiles.len() {
            assert!(
                covered_tiles.contains(&i),
                "Tile {} should be covered by an instance",
                i
            );
        }

        // Verify no duplicate coverage
        assert_eq!(
            covered_tiles.len(),
            shape_data.instances.len(),
            "Each tile should be covered exactly once"
        );
    }
}

/// Test that pentagon and hexagon shapes are correctly identified
#[test]
fn test_shape_type_classification() {
    for subdivision in 2..=5 {
        let hexasphere = Hexasphere::new(1.0, subdivision, 1.0);
        let shape_data = hexasphere.get_shape_instances();

        let mut pentagon_shapes = 0;
        let mut hexagon_shapes = 0;

        for shape in &shape_data.shapes {
            match shape.sides {
                5 => pentagon_shapes += 1,
                6 => hexagon_shapes += 1,
                n => panic!("Unexpected shape with {} sides", n),
            }
        }

        // Should have at least 1 pentagon shape
        assert!(
            pentagon_shapes >= 1,
            "Should have at least 1 pentagon shape"
        );

        // Most shapes should be hexagons (except for subdivision 1)
        if subdivision > 1 {
            assert!(
                hexagon_shapes > pentagon_shapes,
                "Should have more hexagon shapes than pentagon shapes"
            );
        }
    }
}

/// Test that hexagon-only filtering works correctly
#[test]
fn test_hexagon_only_shapes() {
    for subdivision in 3..=6 {
        let hexasphere = Hexasphere::new(1.0, subdivision, 1.0);
        let hex_data = hexasphere.get_hexagon_shape_instances();

        // All shapes should be hexagons
        for shape in &hex_data.shapes {
            assert_eq!(shape.sides, 6, "All shapes should be hexagons");
        }

        // All instances should reference hexagonal tiles
        for instance in &hex_data.instances {
            let original_tile = &hexasphere.tiles[instance.tile_index];
            assert_eq!(
                original_tile.boundary.len(),
                6,
                "All instances should reference hexagonal tiles"
            );
        }

        // Should have fewer instances than total tiles (excluding pentagons)
        let expected_hexagons = hexasphere
            .tiles
            .iter()
            .filter(|t| t.boundary.len() == 6)
            .count();
        assert_eq!(
            hex_data.instances.len(),
            expected_hexagons,
            "Should have exactly as many instances as hexagonal tiles"
        );
    }
}

/// Test that uniform shape approach produces consistent results
#[test]
fn test_uniform_shape_instances() {
    for subdivision in 2..=5 {
        let hexasphere = Hexasphere::new(1.0, subdivision, 1.0);
        let (uniform_shape, instances) = hexasphere.get_uniform_shape_instances();

        // Should have exactly one shape (hexagon)
        assert_eq!(uniform_shape.sides, 6, "Uniform shape should be a hexagon");

        // All instances should reference the same shape (index 0)
        for instance in &instances {
            assert_eq!(
                instance.shape_index, 0,
                "All instances should reference the uniform shape"
            );
        }

        // Should have an instance for every tile that has a valid orientation
        let tiles_with_orientation = hexasphere
            .tiles
            .iter()
            .filter(|t| t.get_orientation().is_some())
            .count();
        assert_eq!(
            instances.len(),
            tiles_with_orientation,
            "Should have instance for every tile with valid orientation"
        );
    }
}

/// Test compression ratios to ensure instancing provides expected benefits
#[test]
fn test_compression_ratios() {
    for subdivision in 3..=8 {
        let hexasphere = Hexasphere::new(1.0, subdivision, 1.0);
        let shape_data = hexasphere.get_shape_instances();

        let compression_ratio = hexasphere.tiles.len() as f64 / shape_data.shapes.len() as f64;

        // Compression should improve with higher subdivision levels
        assert!(
            compression_ratio >= 2.0,
            "Should achieve at least 2x compression at subdivision {}",
            subdivision
        );

        if subdivision >= 5 {
            assert!(
                compression_ratio >= 5.0,
                "Should achieve at least 5x compression at subdivision {}",
                subdivision
            );
        }

        println!(
            "Subdivision {}: {} tiles → {} shapes = {:.1}x compression",
            subdivision,
            hexasphere.tiles.len(),
            shape_data.shapes.len(),
            compression_ratio
        );
    }
}

/// Test that shape analysis produces the same results as shape instances
#[test]
fn test_shape_analysis_consistency() {
    for subdivision in 2..=6 {
        let hexasphere = Hexasphere::new(1.0, subdivision, 1.0);

        // Get results from shape instances
        let shape_data = hexasphere.get_shape_instances();

        // Get results from shape analysis
        let tiles = &hexasphere.tiles;
        let centers: Vec<_> = tiles.iter().map(|t| t.center_point.clone()).collect();
        let mut analyzer = ShapeAnalyzer::new();
        analyzer.analyze_tiles(tiles, &centers);

        // Should produce the same unique count
        assert_eq!(
            shape_data.shapes.len(),
            analyzer.unique_shape_count(),
            "Shape instances and analysis should agree on unique count"
        );
    }
}

/// Test that all instances have valid orientations
#[test]
fn test_instance_orientations() {
    for subdivision in 2..=5 {
        let hexasphere = Hexasphere::new(1.0, subdivision, 1.0);
        let shape_data = hexasphere.get_shape_instances();

        for instance in &shape_data.instances {
            // Check that orientation vectors are normalized
            let right_len = (instance.orientation.right.x.powi(2)
                + instance.orientation.right.y.powi(2)
                + instance.orientation.right.z.powi(2))
            .sqrt();
            assert!(
                (right_len - 1.0).abs() < 0.01,
                "Right vector should be normalized"
            );

            let up_len = (instance.orientation.up.x.powi(2)
                + instance.orientation.up.y.powi(2)
                + instance.orientation.up.z.powi(2))
            .sqrt();
            assert!(
                (up_len - 1.0).abs() < 0.01,
                "Up vector should be normalized"
            );

            let forward_len = (instance.orientation.forward.x.powi(2)
                + instance.orientation.forward.y.powi(2)
                + instance.orientation.forward.z.powi(2))
            .sqrt();
            assert!(
                (forward_len - 1.0).abs() < 0.01,
                "Forward vector should be normalized"
            );
        }
    }
}

/// Test normalized shape instances functionality
#[test]
fn test_normalized_shapes() {
    let hs = Hexasphere::new(1.0, 2, 1.0);
    let max_shapes = 10;
    let tolerance = 0.05;
    let shape_data = hs.get_normalized_shape_instances(max_shapes, tolerance);

    // Should have shapes and instances
    assert!(!shape_data.shapes.is_empty());
    assert!(!shape_data.instances.is_empty());

    // All shapes should be flattened to XY-plane (Z ≈ 0)
    for shape in &shape_data.shapes {
        for vertex in &shape.vertices {
            assert!(
                vertex.z.abs() < 1e-10,
                "Vertex Z={} should be near zero for normalized shape",
                vertex.z
            );
        }

        // Should still have proper geometry in XY
        assert!(shape.vertices.len() >= 5); // At least pentagon
        assert!(shape.radius > 0.0);
    }
}

/// Test normalized tile shape creation
#[test]
fn test_normalized_tile_shape() {
    let hs = Hexasphere::new(1.0, 2, 1.0);
    let tile = &hs.tiles[0];

    if let Some(orientation) = tile.get_orientation() {
        let normalized_shape = TileShape::from_tile_normalized(tile, &orientation);

        // All vertices should have Z ≈ 0
        for vertex in &normalized_shape.vertices {
            assert!(
                vertex.z.abs() < 1e-10,
                "Normalized vertex Z={} should be near zero",
                vertex.z
            );
        }

        // Should preserve basic properties
        assert_eq!(normalized_shape.sides, tile.boundary.len());
        assert!(normalized_shape.radius > 0.0);
        assert_eq!(normalized_shape.vertices.len(), tile.boundary.len());
    }
}

/// Test edge cases for normalized shape instances
#[test]
fn test_normalized_shape_edge_cases() {
    let hexasphere = Hexasphere::new(1.0, 10, 0.95);
    let natural_shapes = hexasphere.get_shape_instances();

    // Test 1: Request more shapes than available (should return natural shapes)
    let shape_data = hexasphere.get_normalized_shape_instances(1000, 0.05);
    assert_eq!(shape_data.shapes.len(), natural_shapes.shapes.len());

    // Test 2: Zero tolerance (should create many clusters)
    let shape_data = hexasphere.get_normalized_shape_instances(50, 0.0);
    assert_eq!(shape_data.instances.len(), hexasphere.tiles.len());

    // Test 3: Very high tolerance (should create few clusters)
    let shape_data = hexasphere.get_normalized_shape_instances(20, 0.5);
    assert!(shape_data.shapes.len() <= 25); // Should cluster aggressively
    assert_eq!(shape_data.instances.len(), hexasphere.tiles.len());

    // Test 4: Minimum shapes (just pentagons + 1 hexagon)
    let shape_data = hexasphere.get_normalized_shape_instances(13, 1.0);
    let pentagon_count = shape_data.shapes.iter().filter(|s| s.sides == 5).count();
    assert!(pentagon_count > 0); // Should preserve pentagons
    assert!(shape_data.shapes.len() <= 13);
    assert_eq!(shape_data.instances.len(), hexasphere.tiles.len());
}

/// Test that the tile count formula 10n² + 2 is correct for reasonable subdivision levels
#[test]
fn test_tile_count_formula_basic() {
    let test_cases = [
        (0, 12),    // Special case
        (1, 12),    // 10(1) + 2 = 12
        (2, 42),    // 10(4) + 2 = 42
        (3, 92),    // 10(9) + 2 = 92
        (4, 162),   // 10(16) + 2 = 162
        (5, 252),   // 10(25) + 2 = 252
        (10, 1002), // 10(100) + 2 = 1002
    ];

    for (n, expected) in test_cases {
        let hexasphere = Hexasphere::new(1.0, n, 1.0);
        let actual_count = hexasphere.tiles.len();

        assert_eq!(
            actual_count, expected,
            "Tile count formula should be 10n² + 2 for subdivision level {}",
            n
        );
    }
}

/// Test that hexagon count follows the expected pattern
#[test]
fn test_hexagon_count_pattern() {
    for level in [1, 2, 3, 4, 5, 8] {
        let hexasphere = Hexasphere::new(1.0, level, 1.0);
        let hexagon_count = hexasphere
            .tiles
            .iter()
            .filter(|tile| tile.boundary.len() == 6)
            .count();

        let total_tiles = hexasphere.tiles.len();
        let expected_hexagons = total_tiles - 12; // Total minus 12 pentagons

        assert_eq!(
            hexagon_count, expected_hexagons,
            "Hexagon count should be total tiles minus 12 at level {}",
            level
        );
    }
}

/// Test that tiles are either pentagons or hexagons
#[test]
fn test_tile_types_only() {
    for level in [2, 3, 4, 5] {
        let hexasphere = Hexasphere::new(1.0, level, 1.0);

        for (i, tile) in hexasphere.tiles.iter().enumerate() {
            let sides = tile.boundary.len();
            assert!(
                sides == 5 || sides == 6,
                "Tile {} at level {} should have 5 or 6 sides, found {}",
                i,
                level,
                sides
            );
        }
    }
}

/// Test basic generation and geometric properties
#[test]
fn test_basic_generation() {
    let level = 6;
    let hexasphere = Hexasphere::new(1.0, level, 1.0);

    println!(
        "Testing {} tiles at subdivision {}",
        hexasphere.tiles.len(),
        level
    );

    let mut pentagon_count = 0;
    let mut hexagon_count = 0;

    for tile in &hexasphere.tiles {
        let sides = tile.boundary.len();

        match sides {
            5 => pentagon_count += 1,
            6 => hexagon_count += 1,
            n => panic!("Unexpected tile with {} sides", n),
        }

        // Check that tile has valid center
        let center_distance = (tile.center_point.x.powi(2)
            + tile.center_point.y.powi(2)
            + tile.center_point.z.powi(2))
        .sqrt();
        assert!(
            (center_distance - 1.0).abs() < 0.1,
            "Tile center should be approximately on unit sphere"
        );

        // Check that boundary points are reasonable
        for vertex in &tile.boundary {
            let vertex_distance = (vertex.x.powi(2) + vertex.y.powi(2) + vertex.z.powi(2)).sqrt();
            assert!(
                vertex_distance > 0.5 && vertex_distance < 1.5,
                "Boundary vertices should be near unit sphere"
            );
        }
    }

    assert_eq!(pentagon_count, 12, "Should have exactly 12 pentagons");
    assert_eq!(hexagon_count + pentagon_count, hexasphere.tiles.len());

    println!("Pentagons: {}, Hexagons: {}", pentagon_count, hexagon_count);
}