use geotiles::Hexasphere;
use std::time::Instant;

/// Test that the tile count formula 10n² + 2 is correct
#[test]
fn test_tile_count_formula() {
    for n in 0..=30 {
        let hexasphere = Hexasphere::new(1.0, n, 1.0);
        let actual_count = hexasphere.tiles.len();
        let expected_count = if n == 0 { 12 } else { 10 * n * n + 2 };
        
        assert_eq!(actual_count, expected_count,
            "Tile count formula should be 10n² + 2 for subdivision level {}", n);
    }
}

/// Test specific tile counts mentioned in issue
#[test]
fn test_reported_tile_counts() {
    // Test the specific numbers mentioned by the user
    let hs_20 = Hexasphere::new(1.0, 20, 1.0);
    assert_eq!(hs_20.tiles.len(), 4002, "Level 20 should have 4002 tiles");
    
    let hs_30 = Hexasphere::new(1.0, 30, 1.0);
    assert_eq!(hs_30.tiles.len(), 9002, "Level 30 should have 9002 tiles");
}

/// Performance test comparing traditional vs shape instancing approach
#[test]
fn test_performance_comparison() {
    for subdivision in [5, 8, 10] {
        println!("\n=== Performance Comparison: Subdivision {} ===", subdivision);
        
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
        println!("Compression: {:.1}x", 
            hexasphere.tiles.len() as f64 / shape_data.shapes.len() as f64);
        
        // Shape instancing should be at least as fast as traditional
        // (though they're both very fast at these levels)
        assert!(instancing_time <= traditional_time * 2,
            "Shape instancing should not be significantly slower than traditional approach");
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
        
        println!("{:5} | {:6} | {:8?} | {:6} | {:9.1}x",
            level, hexasphere.tiles.len(), generation_time, 
            shape_data.shapes.len(), compression);
        
        // Time should grow reasonably (quadratically with tile count)
        if level > 1 {
            let time_ratio = generation_time.as_nanos() as f64 / prev_time.as_nanos() as f64;
            // Allow for some variance in timing
            assert!(time_ratio < 50.0, 
                "Time growth should be reasonable between levels");
        }
        
        prev_time = generation_time;
    }
}

/// Test memory efficiency at different subdivision levels
#[test]
fn test_memory_efficiency_scaling() {
    println!("\n=== Memory Efficiency Analysis ===");
    println!("Level | Traditional (KB) | Instanced (KB) | Savings");
    println!("------|------------------|----------------|--------");
    
    for level in [5, 10, 15, 20] {
        let hexasphere = Hexasphere::new(1.0, level, 1.0);
        let shape_data = hexasphere.get_shape_instances();
        
        // Estimate memory usage (simplified)
        let vertices_per_tile = 6;
        let bytes_per_vertex = 12; // 3 floats × 4 bytes
        
        // Traditional: all tile geometry
        let traditional_kb = (hexasphere.tiles.len() * vertices_per_tile * bytes_per_vertex) / 1024;
        
        // Instanced: unique shapes + instance data
        let shape_kb = (shape_data.shapes.len() * vertices_per_tile * bytes_per_vertex) / 1024;
        let instance_kb = (shape_data.instances.len() * 32) / 1024; // Transform matrix
        let instanced_kb = shape_kb + instance_kb;
        
        let savings = 100.0 * (1.0 - instanced_kb as f64 / traditional_kb as f64);
        
        println!("{:5} | {:16} | {:14} | {:6.1}%",
            level, traditional_kb, instanced_kb, savings);
        
        assert!(savings > 0.0, "Shape instancing should provide memory savings");
        
        if level >= 15 {
            assert!(savings > 80.0, 
                "Should achieve significant savings at high subdivision levels");
        }
    }
}

/// Test that pentagon count is always 12
#[test]
fn test_pentagon_count_invariant() {
    for level in 0..=20 {
        let hexasphere = Hexasphere::new(1.0, level, 1.0);
        let pentagon_count = hexasphere.tiles.iter()
            .filter(|tile| tile.boundary.len() == 5)
            .count();
        
        assert_eq!(pentagon_count, 12,
            "Should always have exactly 12 pentagons at level {}", level);
    }
}

/// Test that hexagon count follows the expected pattern
#[test]
fn test_hexagon_count_pattern() {
    for level in 1..=15 {
        let hexasphere = Hexasphere::new(1.0, level, 1.0);
        let hexagon_count = hexasphere.tiles.iter()
            .filter(|tile| tile.boundary.len() == 6)
            .count();
        
        let total_tiles = hexasphere.tiles.len();
        let expected_hexagons = total_tiles - 12; // Total minus 12 pentagons
        
        assert_eq!(hexagon_count, expected_hexagons,
            "Hexagon count should be total tiles minus 12 at level {}", level);
    }
}

/// Benchmark shape generation vs traditional approach
#[test]
fn test_generation_efficiency() {
    // Test at a moderately high subdivision level
    let level = 12;
    let iterations = 5;
    
    println!("\n=== Generation Efficiency Test (Level {}) ===", level);
    
    // Benchmark traditional approach
    let mut traditional_times = Vec::new();
    for _ in 0..iterations {
        let start = Instant::now();
        let _hexasphere = Hexasphere::new(1.0, level, 1.0);
        traditional_times.push(start.elapsed());
    }
    
    // Benchmark with shape instancing
    let mut instancing_times = Vec::new();
    for _ in 0..iterations {
        let hexasphere = Hexasphere::new(1.0, level, 1.0);
        let start = Instant::now();
        let _shape_data = hexasphere.get_shape_instances();
        instancing_times.push(start.elapsed());
    }
    
    let avg_traditional = traditional_times.iter().sum::<std::time::Duration>() / iterations as u32;
    let avg_instancing = instancing_times.iter().sum::<std::time::Duration>() / iterations as u32;
    
    println!("Average traditional generation: {:?}", avg_traditional);
    println!("Average shape instancing: {:?}", avg_instancing);
    
    // The actual hexasphere generation should dominate, so instancing should be much faster
    // when it's called on an already-generated hexasphere
    println!("Shape instancing is {:.1}x faster", 
        avg_traditional.as_nanos() as f64 / avg_instancing.as_nanos() as f64);
}