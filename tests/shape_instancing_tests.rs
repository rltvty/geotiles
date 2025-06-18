use geotiles::hexasphere::shape_instances::{ShapeInstanceData, TileInstance, TileShape};
use geotiles::hexasphere::{Hexasphere, ShapeAnalyzer};
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
        for (i, instance) in shape_data.instances.iter().enumerate() {
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

/// Test memory efficiency of shape instancing
#[test]
fn test_memory_efficiency() {
    for subdivision in [5, 10, 15] {
        let hexasphere = Hexasphere::new(1.0, subdivision, 1.0);
        let shape_data = hexasphere.get_shape_instances();

        // Estimate memory usage
        let vertices_per_tile = 6; // Average
        let bytes_per_vertex = 3 * 4; // 3 floats × 4 bytes

        // Traditional approach: store all tile vertices
        let traditional_memory = hexasphere.tiles.len() * vertices_per_tile * bytes_per_vertex;

        // Shape instancing: shapes + instance transforms
        let shape_memory = shape_data.shapes.len() * vertices_per_tile * bytes_per_vertex;
        let instance_memory = shape_data.instances.len() * 32; // Transform data
        let instanced_memory = shape_memory + instance_memory;

        let memory_savings = traditional_memory as f64 / instanced_memory as f64;

        assert!(
            memory_savings > 1.0,
            "Shape instancing should reduce memory usage"
        );

        if subdivision >= 10 {
            assert!(
                memory_savings > 5.0,
                "Should achieve significant memory savings at high subdivision levels"
            );
        }

        println!(
            "Subdivision {}: {:.1}x memory reduction ({} → {} bytes)",
            subdivision, memory_savings, traditional_memory, instanced_memory
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
