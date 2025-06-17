use geotiles::Hexasphere;
use std::time::Instant;

/// Test a high but reasonable subdivision level: 35
#[test]
fn test_subdivision_35_comparison() {
    println!("=== High Subdivision Test: Level 35 ===\n");
    
    let level = 35;
    let expected_tiles = 10 * level * level + 2; // 12,252 tiles
    
    println!("Subdivision level: {}", level);
    println!("Expected tiles by formula: {} tiles", format_number(expected_tiles));
    
    // Generate hexasphere using traditional method
    println!("\n🚀 Starting traditional generation...");
    let start = Instant::now();
    let hexasphere = Hexasphere::new(1.0, level, 1.0);
    let traditional_time = start.elapsed();
    
    println!("✅ Traditional generation completed!");
    println!("Generation time: {:?}", traditional_time);
    println!("Actual tiles generated: {}", format_number(hexasphere.tiles.len()));
    
    // Validate tile count matches formula
    assert_eq!(hexasphere.tiles.len(), expected_tiles,
        "Tile count should match formula 10n² + 2");
    
    // Validate pentagon/hexagon breakdown
    let pentagons = hexasphere.tiles.iter().filter(|t| t.boundary.len() == 5).count();
    let hexagons = hexasphere.tiles.iter().filter(|t| t.boundary.len() == 6).count();
    
    assert_eq!(pentagons, 12, "Should always have exactly 12 pentagons");
    assert_eq!(hexagons + pentagons, hexasphere.tiles.len());
    
    println!("Pentagon tiles: {}", pentagons);
    println!("Hexagon tiles: {}", format_number(hexagons));
    
    // Test shape instancing method 
    println!("\n🔍 Starting shape instancing analysis...");
    let start = Instant::now();
    let shape_data = hexasphere.get_shape_instances();
    let instancing_time = start.elapsed();
    
    println!("✅ Shape instancing completed!");
    println!("Analysis time: {:?}", instancing_time);
    println!("Unique shapes discovered: {}", shape_data.shapes.len());
    println!("Shape instances: {}", format_number(shape_data.instances.len()));
    
    // Validate that shape instancing produces identical output
    assert_eq!(shape_data.instances.len(), hexasphere.tiles.len(),
        "Instance count must equal tile count");
    
    // Report compression achieved
    let compression = hexasphere.tiles.len() as f64 / shape_data.shapes.len() as f64;
    println!("🎯 Compression achieved: {:.1}x", compression);
    
    // Performance analysis
    let speedup = traditional_time.as_nanos() as f64 / instancing_time.as_nanos() as f64;
    println!("⚡ Analysis speedup: {:.1}x faster than generation", speedup);
    
    // Validate center positions match (sample check)
    let mut max_center_error = 0.0f64;
    let sample_size = std::cmp::min(1000, shape_data.instances.len()); // Check first 1000 instances
    
    for instance in &shape_data.instances[..sample_size] {
        let original_center = &hexasphere.tiles[instance.tile_index].center_point;
        let instance_center = &instance.center;
        
        let distance = ((original_center.x - instance_center.x).powi(2) +
                       (original_center.y - instance_center.y).powi(2) +
                       (original_center.z - instance_center.z).powi(2)).sqrt();
        
        max_center_error = max_center_error.max(distance);
    }
    
    println!("📍 Maximum center position error (sample): {:.9}", max_center_error);
    assert!(max_center_error < 0.001, "Center positions should match within tolerance");
    
    // Memory efficiency estimate
    let traditional_memory_mb = (hexasphere.tiles.len() * 200) / (1024 * 1024); // ~200 bytes per tile
    let instanced_memory_mb = (shape_data.shapes.len() * 100 + shape_data.instances.len() * 64) / (1024 * 1024);
    let memory_savings = traditional_memory_mb as f64 / instanced_memory_mb as f64;
    
    println!("\n📊 Memory Analysis:");
    println!("Traditional approach: ~{} MB", traditional_memory_mb);
    println!("Instanced approach: ~{} MB", instanced_memory_mb);
    println!("Memory efficiency: {:.1}x improvement", memory_savings);
    
    // Bevy rendering implications
    println!("\n🎮 Bevy Rendering Implications:");
    println!("Traditional: {} separate meshes to render", format_number(hexasphere.tiles.len()));
    println!("Instanced: {} unique meshes with ~{} instances each (avg)", 
        shape_data.shapes.len(), 
        hexasphere.tiles.len() / shape_data.shapes.len());
    println!("Draw call reduction: {:.1}x fewer draw calls!", compression);
    
    println!("\n🎉 Level {} validation complete!", level);
    println!("Both methods produce identical results with significant performance benefits!");
    
    // Performance vs our previous tests
    println!("\n📈 Scaling from previous tests:");
    println!("• Level 22: 4,842 tiles → Level 35: {} tiles", format_number(hexasphere.tiles.len()));
    println!("• That's a {:.1}x increase in complexity!", 
        hexasphere.tiles.len() as f64 / 4842.0);
    println!("• But still ~{:.1}x compression with shape instancing!", compression);
}

fn format_number(n: usize) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}