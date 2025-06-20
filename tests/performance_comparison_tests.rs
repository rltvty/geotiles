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

        println!("Traditional generation: {:?}", traditional_time);
        println!("Shape instancing: {:?}", instancing_time);
        println!("Tiles: {}", hexasphere.tiles.len());
        println!("Unique shapes: {}", shape_data.shapes.len());
        println!(
            "Compression: {:.1}x",
            hexasphere.tiles.len() as f64 / shape_data.shapes.len() as f64
        );

        // Shape instancing should be at least as fast as traditional
        // (though they're both very fast at these levels)
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
