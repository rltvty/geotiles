use geotiles::Hexasphere;
use std::time::Instant;

/// Performance test comparing traditional vs shape instancing approach
#[test]
fn test_performance_comparison() {
    for subdivision in [5, 8, 10] {
        println!(
            "\n=== Performance Comparison: Subdivision {} ===",
            subdivision
        );

        // Time traditional approach (full hexasphere generation)
        let start = Instant::now();
        let hexasphere = Hexasphere::new(1.0, subdivision, 1.0);
        let traditional_time = start.elapsed();

        // Time shape instancing approach
        let start = Instant::now();
        let shape_data = hexasphere.get_shape_instances();
        let instancing_time = start.elapsed();

        // Time normalized shape instancing
        let start = Instant::now();
        let _normalized_data = hexasphere.get_normalized_shape_instances(50, 0.05);
        let normalized_time = start.elapsed();

        println!("Traditional generation: {:?}", traditional_time);
        println!("Shape instancing: {:?}", instancing_time);
        println!("Normalized instancing: {:?}", normalized_time);
        println!("Tiles: {}", hexasphere.tiles.len());
        println!("Unique shapes: {}", shape_data.shapes.len());
        println!(
            "Compression: {:.1}x",
            hexasphere.tiles.len() as f64 / shape_data.shapes.len() as f64
        );

        // Shape instancing should be at least as fast as traditional
        assert!(
            instancing_time <= traditional_time * 2,
            "Shape instancing should not be significantly slower than traditional approach"
        );
    }
}

/// Test scaling behavior at higher subdivision levels
#[test]
fn test_scaling_behavior() {
    let levels = [1, 2, 3, 4, 5, 8, 10];
    let mut prev_time = std::time::Duration::from_nanos(0);

    println!("\n=== Scaling Analysis ===");
    println!("Level | Tiles  | Time     | Shapes | Compression");
    println!("------|--------|----------|--------|------------");

    for &level in &levels {
        let start = Instant::now();
        let hexasphere = Hexasphere::new(1.0, level, 1.0);
        let generation_time = start.elapsed();

        let shape_data = hexasphere.get_shape_instances();
        let compression = hexasphere.tiles.len() as f64 / shape_data.shapes.len() as f64;

        println!(
            "{:5} | {:6} | {:8?} | {:6} | {:9.1}x",
            level,
            hexasphere.tiles.len(),
            generation_time,
            shape_data.shapes.len(),
            compression
        );

        // Time should grow reasonably (quadratically with tile count)
        if level > 1 {
            let time_ratio = generation_time.as_nanos() as f64 / prev_time.as_nanos() as f64;
            // Allow for some variance in timing
            assert!(
                time_ratio < 50.0,
                "Time growth should be reasonable between levels"
            );
        }

        prev_time = generation_time;
    }
}

/// Performance benchmark for the requested levels
#[test]
fn test_performance_scaling() {
    println!("\n=== Performance Scaling Test ===");

    let test_levels = [4, 6, 8, 10, 12];
    let mut previous_time = std::time::Duration::from_nanos(1);

    println!("Level | Tiles  | Time     | Time Ratio | Tiles/sec");
    println!("------|--------|----------|------------|----------");

    for level in test_levels {
        let expected_tiles = 10 * level * level + 2;

        let start = Instant::now();
        let hexasphere = Hexasphere::new(1.0, level, 1.0);
        let generation_time = start.elapsed();

        assert_eq!(hexasphere.tiles.len(), expected_tiles);

        let time_ratio = generation_time.as_nanos() as f64 / previous_time.as_nanos() as f64;
        let tiles_per_sec = hexasphere.tiles.len() as f64 / generation_time.as_secs_f64();

        println!(
            "{:5} | {:6} | {:8?} | {:9.1}x | {:8.0}",
            level,
            hexasphere.tiles.len(),
            generation_time,
            time_ratio,
            tiles_per_sec
        );

        // Time should grow reasonably (not exponentially)
        if level > 4 {
            assert!(
                time_ratio < 10.0,
                "Time growth should be reasonable between consecutive levels"
            );
        }

        previous_time = generation_time;
    }

    println!("\n✅ Performance scaling looks reasonable");
}

/// Test normalized shape instances performance at different tolerance levels
#[test]
fn test_normalized_performance_with_tolerance() {
    println!("\n=== Normalized Shape Performance vs Tolerance ===");

    let level = 12;
    let hexasphere = Hexasphere::new(1.0, level, 0.95);

    println!("Testing tolerance effects on subdivision level {}", level);

    let tolerances = [0.01, 0.02, 0.05, 0.1, 0.2]; // 1% to 20% tolerance
    let target_shapes = 30;

    println!("Tolerance | Time     | Actual Shapes | Compression");
    println!("----------|----------|---------------|------------");

    for tolerance in tolerances {
        let start = Instant::now();
        let shape_data = hexasphere.get_normalized_shape_instances(target_shapes, tolerance);
        let processing_time = start.elapsed();

        let compression = hexasphere.tiles.len() as f64 / shape_data.shapes.len() as f64;

        println!(
            "{:7.1}% | {:8?} | {:11} | {:9.1}x",
            tolerance * 100.0,
            processing_time,
            shape_data.shapes.len(),
            compression
        );

        // Validate
        assert_eq!(shape_data.instances.len(), hexasphere.tiles.len());
    }

    println!("\n✅ Tolerance performance analysis complete");
}

/// Benchmark normalized vs regular shape instances
#[test]
fn test_normalized_vs_regular_performance() {
    println!("\n=== Normalized vs Regular Shape Performance ===");

    for subdivision in [8, 10, 12] {
        println!("\nSubdivision {}:", subdivision);

        let hexasphere = Hexasphere::new(1.0, subdivision, 1.0);

        // Time regular shape instances
        let start = Instant::now();
        let _regular_data = hexasphere.get_shape_instances();
        let regular_time = start.elapsed();

        // Time normalized shape instances
        let start = Instant::now();
        let _normalized_data = hexasphere.get_normalized_shape_instances(50, 0.05);
        let normalized_time = start.elapsed();

        // Time hexagon-only
        let start = Instant::now();
        let _hex_data = hexasphere.get_hexagon_shape_instances();
        let hex_only_time = start.elapsed();

        // Time uniform approach
        let start = Instant::now();
        let _uniform_data = hexasphere.get_uniform_shape_instances();
        let uniform_time = start.elapsed();

        println!("  Regular shapes: {:?}", regular_time);
        println!("  Normalized shapes: {:?}", normalized_time);
        println!("  Hexagon-only: {:?}", hex_only_time);
        println!("  Uniform: {:?}", uniform_time);

        // All methods should be reasonably fast
        let max_acceptable = std::time::Duration::from_millis(100);
        assert!(
            regular_time < max_acceptable,
            "Regular shapes should be fast"
        );
        assert!(
            normalized_time < max_acceptable,
            "Normalized shapes should be fast"
        );
        assert!(
            hex_only_time < max_acceptable,
            "Hexagon-only should be fast"
        );
        assert!(uniform_time < max_acceptable, "Uniform should be fast");
    }

    println!("\n✅ Performance comparison complete");
}

/// Test generation consistency across different parameters
#[test]
fn test_generation_consistency() {
    println!("\n=== Testing Generation Consistency ===");

    let level = 8;
    let radius = 1.0;

    // Test different hex_size values produce consistent tile counts
    let hex_sizes = [0.5, 0.8, 0.95, 1.0];
    let expected_tiles = 10 * level * level + 2;

    for hex_size in hex_sizes {
        let hexasphere = Hexasphere::new(radius, level, hex_size);

        assert_eq!(
            hexasphere.tiles.len(),
            expected_tiles,
            "Tile count should be independent of hex_size"
        );

        let pentagons = hexasphere
            .tiles
            .iter()
            .filter(|t| t.boundary.len() == 5)
            .count();
        assert_eq!(
            pentagons, 12,
            "Pentagon count should be independent of hex_size"
        );

        println!(
            "hex_size {}: {} tiles, 12 pentagons ✓",
            hex_size,
            hexasphere.tiles.len()
        );
    }

    // Test different radii produce consistent tile counts
    let radii = [0.5, 1.0, 2.0, 10.0];

    for radius in radii {
        let hexasphere = Hexasphere::new(radius, level, 1.0);

        assert_eq!(
            hexasphere.tiles.len(),
            expected_tiles,
            "Tile count should be independent of radius"
        );

        // Check that radius affects actual coordinates
        let max_distance = hexasphere
            .tiles
            .iter()
            .map(|t| {
                (t.center_point.x.powi(2) + t.center_point.y.powi(2) + t.center_point.z.powi(2))
                    .sqrt()
            })
            .fold(0.0f64, |a, b| a.max(b));

        assert!(
            (max_distance - radius).abs() < 0.1,
            "Max distance should be approximately equal to radius"
        );

        println!(
            "radius {}: {} tiles, max distance {:.3} ✓",
            radius,
            hexasphere.tiles.len(),
            max_distance
        );
    }

    println!("✅ Generation consistency validated");
}
