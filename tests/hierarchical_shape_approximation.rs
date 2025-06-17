use geotiles::Hexasphere;
use std::time::Instant;

/// Test the new hierarchical shape approximation system
#[test]
fn test_hierarchical_shape_approximation() {
    println!("=== Hierarchical Shape Approximation Test ===\n");
    
    let level = 15;
    let hexasphere = Hexasphere::new(1.0, level, 0.95);
    
    println!("Testing subdivision level {} ({} tiles)", level, hexasphere.tiles.len());
    
    // Get natural shape instances for comparison
    let natural_shapes = hexasphere.get_shape_instances();
    let natural_compression = hexasphere.tiles.len() as f64 / natural_shapes.shapes.len() as f64;
    
    println!("Natural instancing: {} shapes ({:.1}x compression)", 
        natural_shapes.shapes.len(), natural_compression);
    
    // Test different approximation levels
    let approximation_tests = [
        (25, 0.02, "Ultra performance (Mobile VR)"),
        (50, 0.03, "High performance (Gaming)"),
        (100, 0.05, "Balanced (Desktop)"),
    ];
    
    println!("\nApproximation | Shapes | Total Compression | Error Avg | Error Max | Use Case");
    println!("--------------|--------|-------------------|-----------|-----------|------------------");
    
    for (max_shapes, tolerance, description) in approximation_tests {
        let start = Instant::now();
        let (shapes, instances, stats) = hexasphere.get_simplified_shapes(max_shapes, tolerance);
        let processing_time = start.elapsed();
        
        let total_compression = hexasphere.tiles.len() as f64 / shapes.len() as f64;
        
        println!("{:12} | {:6} | {:15.1}x | {:7.2}% | {:7.2}% | {}",
            max_shapes,
            shapes.len(),
            total_compression,
            stats.average_error * 100.0,
            stats.max_error * 100.0,
            description
        );
        
        // Validate results
        assert_eq!(instances.len(), hexasphere.tiles.len(),
            "Should have same number of instances as tiles");
        assert!(shapes.len() <= max_shapes + 5, // Allow some tolerance
            "Should not exceed target by much");
        assert_eq!(stats.pentagon_shapes_preserved, 12,
            "Should preserve all 12 pentagon shapes");
        
        // Verify all tiles are covered
        let covered_tiles: std::collections::HashSet<_> = instances
            .iter()
            .map(|inst| inst.tile_index)
            .collect();
        assert_eq!(covered_tiles.len(), hexasphere.tiles.len(),
            "All tiles should be covered exactly once");
        
        println!("         Processing time: {:?}, Compression improvement: {:.1}x",
            processing_time, stats.compression_ratio);
    }
    
    println!("\n✅ Hierarchical approximation working correctly!");
}

/// Test extreme approximation scenarios
#[test]
fn test_extreme_approximation() {
    println!("=== Extreme Approximation Test ===\n");
    
    let level = 20;
    let hexasphere = Hexasphere::new(1.0, level, 0.95);
    let natural_shapes = hexasphere.get_shape_instances();
    
    println!("High-detail sphere: {} tiles, {} natural shapes", 
        hexasphere.tiles.len(), natural_shapes.shapes.len());
    
    // Test very aggressive approximation
    let extreme_tests = [15, 20, 30];
    
    for target in extreme_tests {
        println!("\n--- Ultra-aggressive approximation: {} shapes ---", target);
        
        let (shapes, instances, stats) = hexasphere.get_simplified_shapes(target, 0.1); // 10% tolerance
        
        let total_compression = hexasphere.tiles.len() as f64 / shapes.len() as f64;
        let natural_compression = hexasphere.tiles.len() as f64 / natural_shapes.shapes.len() as f64;
        let additional_compression = total_compression / natural_compression;
        
        println!("Results:");
        println!("  Simplified shapes: {}", shapes.len());
        println!("  Total compression: {:.0}x", total_compression);
        println!("  Additional compression: {:.1}x over natural", additional_compression);
        println!("  Average error: {:.2}%", stats.average_error * 100.0);
        println!("  Max error: {:.2}%", stats.max_error * 100.0);
        
        // GPU performance implications
        let traditional_draw_calls = hexasphere.tiles.len();
        let simplified_draw_calls = shapes.len();
        let draw_call_reduction = traditional_draw_calls as f64 / simplified_draw_calls as f64;
        
        println!("  GPU draw calls: {} → {} ({:.0}x reduction)", 
            traditional_draw_calls, simplified_draw_calls, draw_call_reduction);
        
        // Memory estimates
        let traditional_memory = traditional_draw_calls * 200; // bytes per mesh
        let simplified_memory = simplified_draw_calls * 100 + instances.len() * 32; // shapes + instances
        let memory_efficiency = traditional_memory as f64 / simplified_memory as f64;
        
        println!("  Memory efficiency: {:.1}x improvement", memory_efficiency);
        
        // Validate
        assert_eq!(instances.len(), hexasphere.tiles.len());
        assert!(shapes.len() <= target + 2); // Allow small tolerance
        
        if total_compression > 100.0 {
            println!("  🚀 Incredible performance for mobile/VR applications!");
        }
    }
    
    println!("\n✅ Extreme approximation validation complete!");
}

/// Test different tolerance levels
#[test]
fn test_tolerance_effects() {
    println!("=== Tolerance Effects Test ===\n");
    
    let level = 12;
    let hexasphere = Hexasphere::new(1.0, level, 0.95);
    
    println!("Testing tolerance effects on subdivision level {}", level);
    
    let tolerances = [0.01, 0.02, 0.05, 0.1, 0.2]; // 1% to 20% tolerance
    let target_shapes = 30;
    
    println!("Tolerance | Actual Shapes | Avg Error | Max Error | Compression");
    println!("----------|---------------|-----------|-----------|------------");
    
    for tolerance in tolerances {
        let (shapes, instances, stats) = hexasphere.get_simplified_shapes(target_shapes, tolerance);
        let compression = hexasphere.tiles.len() as f64 / shapes.len() as f64;
        
        println!("{:7.1}% | {:11} | {:7.2}% | {:7.2}% | {:9.1}x",
            tolerance * 100.0,
            shapes.len(),
            stats.average_error * 100.0,
            stats.max_error * 100.0,
            compression
        );
        
        // Validate
        assert_eq!(instances.len(), hexasphere.tiles.len());
        assert!(stats.average_error <= tolerance * 1.5, // Allow some margin
            "Average error should be within tolerance range");
    }
    
    println!("\n✅ Tolerance effects validation complete!");
}

/// Test scalability across different subdivision levels
#[test]
fn test_scalability_across_levels() {
    println!("=== Scalability Test Across Subdivision Levels ===\n");
    
    let test_levels = [10, 15, 20, 25];
    let target_shapes = 50;
    let tolerance = 0.03; // 3% tolerance
    
    println!("Level | Tiles | Natural | Simplified | Total Compression | Processing");
    println!("------|-------|---------|------------|-------------------|------------");
    
    for level in test_levels {
        let hexasphere = Hexasphere::new(1.0, level, 0.95);
        let natural_shapes = hexasphere.get_shape_instances();
        
        let start = Instant::now();
        let (shapes, instances, stats) = hexasphere.get_simplified_shapes(target_shapes, tolerance);
        let processing_time = start.elapsed();
        
        let total_compression = hexasphere.tiles.len() as f64 / shapes.len() as f64;
        
        println!("{:5} | {:5} | {:7} | {:8} | {:15.1}x | {:?}",
            level,
            hexasphere.tiles.len(),
            natural_shapes.shapes.len(),
            shapes.len(),
            total_compression,
            processing_time
        );
        
        // Validate
        assert_eq!(instances.len(), hexasphere.tiles.len());
        assert!(shapes.len() <= target_shapes + 5);
        assert!(processing_time.as_millis() < 1000, "Should process quickly");
    }
    
    println!("\n✅ Scalability validation complete!");
}

/// Test error edge cases
#[test]
fn test_edge_cases() {
    println!("=== Edge Cases Test ===\n");
    
    let hexasphere = Hexasphere::new(1.0, 10, 0.95);
    let natural_shapes = hexasphere.get_shape_instances();
    
    // Test 1: Request more shapes than available (should return natural shapes)
    println!("Test 1: Request more shapes than available");
    let (shapes, instances, stats) = hexasphere.get_simplified_shapes(1000, 0.05);
    assert_eq!(shapes.len(), natural_shapes.shapes.len());
    assert_eq!(stats.compression_ratio, 1.0);
    assert_eq!(stats.average_error, 0.0);
    println!("✓ Correctly returned natural shapes when target exceeds available");
    
    // Test 2: Zero tolerance (should create many clusters)
    println!("\nTest 2: Zero tolerance");
    let (shapes, instances, stats) = hexasphere.get_simplified_shapes(50, 0.0);
    assert!(stats.average_error < 0.001);
    assert_eq!(instances.len(), hexasphere.tiles.len());
    println!("✓ Zero tolerance works correctly");
    
    // Test 3: Very high tolerance (should create few clusters)
    println!("\nTest 3: Very high tolerance");
    let (shapes, instances, stats) = hexasphere.get_simplified_shapes(20, 0.5);
    assert!(shapes.len() <= 25); // Should cluster aggressively
    assert_eq!(instances.len(), hexasphere.tiles.len());
    println!("✓ High tolerance creates aggressive clustering");
    
    // Test 4: Minimum shapes (just pentagons + 1 hexagon)
    println!("\nTest 4: Minimum hexagon clustering");
    let (shapes, instances, stats) = hexasphere.get_simplified_shapes(13, 1.0); // 12 pentagons + 1 hexagon
    assert_eq!(stats.pentagon_shapes_preserved, 12);
    assert!(shapes.len() >= 13);
    assert_eq!(instances.len(), hexasphere.tiles.len());
    println!("✓ Minimum clustering preserves pentagons");
    
    println!("\n✅ All edge cases handled correctly!");
}