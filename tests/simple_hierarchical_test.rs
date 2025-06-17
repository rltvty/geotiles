use geotiles::Hexasphere;

/// Simple test of the new hierarchical shape approximation
#[test]
fn test_simple_hierarchical_approximation() {
    println!("=== Simple Hierarchical Approximation Test ===\n");
    
    let level = 10;
    let hexasphere = Hexasphere::new(1.0, level, 0.95);
    
    println!("Testing subdivision level {} ({} tiles)", level, hexasphere.tiles.len());
    
    // Get natural shape instances for comparison
    let natural_shapes = hexasphere.get_shape_instances();
    let natural_hexagon_count = natural_shapes.shapes.iter().filter(|s| s.sides == 6).count();
    let natural_pentagon_count = natural_shapes.shapes.iter().filter(|s| s.sides == 5).count();
    let natural_compression = hexasphere.tiles.len() as f64 / natural_shapes.shapes.len() as f64;
    
    println!("Natural instancing: {} total shapes ({} hexagons, {} pentagons) ({:.1}x total compression)", 
        natural_shapes.shapes.len(), natural_hexagon_count, natural_pentagon_count, natural_compression);
    
    // Test hierarchical approximation
    let target_shapes = 25;
    let tolerance = 0.05; // 5% tolerance
    
    let (simplified_shapes, simplified_instances, stats) = hexasphere.get_simplified_shapes(target_shapes, tolerance);
    
    let simplified_hexagon_count = simplified_shapes.iter().filter(|s| s.sides == 6).count();
    let simplified_pentagon_count = simplified_shapes.iter().filter(|s| s.sides == 5).count();
    
    println!("\nHierarchical approximation results:");
    println!("  Target hexagon shapes: {}", target_shapes);
    println!("  Actual hexagon shapes: {}", simplified_hexagon_count);
    println!("  Pentagon shapes: {} (always preserved)", simplified_pentagon_count);
    println!("  Total shapes: {}", simplified_shapes.len());
    println!("  Hexagon compression: {:.1}x ({} → {})", stats.compression_ratio, stats.original_shape_count, stats.simplified_shape_count);
    println!("  Average error (hexagons): {:.2}%", stats.average_error * 100.0);
    println!("  Max error (hexagons): {:.2}%", stats.max_error * 100.0);
    println!("  Pentagon instances preserved: {}", stats.pentagon_shapes_preserved);
    
    // Validate results
    assert_eq!(simplified_instances.len(), hexasphere.tiles.len(),
        "Should have same number of instances as tiles");
    assert!(simplified_hexagon_count <= target_shapes + 5,
        "Should not exceed hexagon target by much");
    assert_eq!(stats.pentagon_shapes_preserved, 12,
        "Should preserve all 12 pentagon shapes");
    
    // Verify all tiles are covered
    let covered_tiles: std::collections::HashSet<_> = simplified_instances
        .iter()
        .map(|inst| inst.tile_index)
        .collect();
    assert_eq!(covered_tiles.len(), hexasphere.tiles.len(),
        "All tiles should be covered exactly once");
    
    // GPU performance implications
    let traditional_draw_calls = hexasphere.tiles.len();
    let hierarchical_draw_calls = simplified_shapes.len();
    let draw_call_reduction = traditional_draw_calls as f64 / hierarchical_draw_calls as f64;
    
    println!("\nGPU Performance Benefits:");
    println!("  Traditional: {} draw calls", traditional_draw_calls);
    println!("  Hierarchical: {} draw calls", hierarchical_draw_calls);
    println!("  Draw call reduction: {:.0}x", draw_call_reduction);
    
    if draw_call_reduction > 30.0 {
        println!("  🚀 Excellent performance for real-time applications!");
    }
    
    println!("\n✅ Hierarchical approximation working correctly!");
}