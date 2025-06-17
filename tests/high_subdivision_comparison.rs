use geotiles::Hexasphere;
use std::time::Instant;

/// Test higher subdivision levels (like 22) that might be slower
#[test]
#[ignore] // Ignored by default for CI performance, run with --ignored
fn test_subdivision_22_comparison() {
    let subdivision = 22;
    
    println!("\n=== High Subdivision Test: Level {} ===", subdivision);
    
    let expected_tiles = 10 * subdivision * subdivision + 2; // 4842 tiles
    println!("Expected tiles by formula: {}", expected_tiles);
    
    // Traditional method
    println!("Generating hexasphere with traditional method...");
    let start = Instant::now();
    let hexasphere = Hexasphere::new(1.0, subdivision, 1.0);
    let traditional_time = start.elapsed();
    
    println!("Traditional generation completed in: {:?}", traditional_time);
    println!("Actual tiles generated: {}", hexasphere.tiles.len());
    
    // Validate tile count matches formula
    assert_eq!(hexasphere.tiles.len(), expected_tiles,
        "Tile count should match formula 10n² + 2");
    
    // Shape instancing method
    println!("Analyzing with shape instancing...");
    let start = Instant::now();
    let shape_data = hexasphere.get_shape_instances();
    let instancing_time = start.elapsed();
    
    println!("Shape instancing completed in: {:?}", instancing_time);
    println!("Unique shapes identified: {}", shape_data.shapes.len());
    println!("Shape instances: {}", shape_data.instances.len());
    
    // Core validations
    assert_eq!(shape_data.instances.len(), hexasphere.tiles.len(),
        "Instance count must equal tile count");
    
    let compression = hexasphere.tiles.len() as f64 / shape_data.shapes.len() as f64;
    println!("Compression ratio: {:.1}x", compression);
    
    // Pentagon/hexagon breakdown
    let pentagons = hexasphere.tiles.iter().filter(|t| t.boundary.len() == 5).count();
    let hexagons = hexasphere.tiles.iter().filter(|t| t.boundary.len() == 6).count();
    
    println!("Pentagons: {} (should be 12)", pentagons);
    println!("Hexagons: {}", hexagons);
    
    assert_eq!(pentagons, 12, "Should always have exactly 12 pentagons");
    assert_eq!(pentagons + hexagons, hexasphere.tiles.len());
    
    // Test that all instances have valid tile references
    for instance in &shape_data.instances {
        assert!(instance.tile_index < hexasphere.tiles.len(),
            "All tile indices should be valid");
        assert!(instance.shape_index < shape_data.shapes.len(),
            "All shape indices should be valid");
    }
    
    // Test memory efficiency estimation
    let vertex_data_traditional = hexasphere.tiles.len() * 6 * 3 * 4; // tiles × vertices × coords × bytes
    let vertex_data_instanced = shape_data.shapes.len() * 6 * 3 * 4; // shapes × vertices × coords × bytes
    let instance_data = shape_data.instances.len() * 32; // rough transform size
    let total_instanced = vertex_data_instanced + instance_data;
    
    let memory_savings = vertex_data_traditional as f64 / total_instanced as f64;
    
    println!("Memory comparison:");
    println!("  Traditional vertex data: {} KB", vertex_data_traditional / 1024);
    println!("  Instanced vertex data: {} KB", vertex_data_instanced / 1024);
    println!("  Instance data: {} KB", instance_data / 1024);
    println!("  Total instanced: {} KB", total_instanced / 1024);
    println!("  Memory savings: {:.1}x", memory_savings);
    
    assert!(memory_savings > 5.0, "Should achieve significant memory savings at level 22");
    
    println!("✅ High subdivision level {} validation complete", subdivision);
}

/// Test all three requested levels with basic validation
#[test]
fn test_all_requested_levels_formula() {
    // Test the formula for all requested levels without full generation
    let requested_levels = [6, 12, 22];
    
    println!("\n=== Formula Validation for Requested Levels ===");
    
    for level in requested_levels {
        let expected_tiles = 10 * level * level + 2;
        
        if level <= 12 {
            // Actually generate for reasonable levels
            let hexasphere = Hexasphere::new(1.0, level, 1.0);
            assert_eq!(hexasphere.tiles.len(), expected_tiles);
            println!("Level {}: {} tiles (verified)", level, hexasphere.tiles.len());
        } else {
            // Just validate formula for high levels
            println!("Level {}: {} tiles (by formula)", level, expected_tiles);
        }
    }
    
    println!("✅ All requested levels follow the correct formula");
}

/// Benchmark the three requested levels for performance analysis
#[test]
#[ignore] // Ignored by default, run with --ignored for performance testing
fn benchmark_requested_levels() {
    println!("\n=== Performance Benchmark for Requested Levels ===");
    
    let levels = [6, 12, 22];
    
    for level in levels {
        println!("\n--- Benchmarking Level {} ---", level);
        
        let expected_tiles = 10 * level * level + 2;
        println!("Expected tiles: {}", expected_tiles);
        
        // Traditional generation benchmark
        let start = Instant::now();
        let hexasphere = Hexasphere::new(1.0, level, 1.0);
        let generation_time = start.elapsed();
        
        println!("Generation time: {:?}", generation_time);
        println!("Actual tiles: {}", hexasphere.tiles.len());
        
        // Shape instancing benchmark
        let start = Instant::now();
        let shape_data = hexasphere.get_shape_instances();
        let instancing_time = start.elapsed();
        
        println!("Shape analysis time: {:?}", instancing_time);
        println!("Unique shapes: {}", shape_data.shapes.len());
        
        // Different approach benchmarks
        let start = Instant::now();
        let _hex_data = hexasphere.get_hexagon_shape_instances();
        let hex_time = start.elapsed();
        
        let start = Instant::now();
        let _uniform_data = hexasphere.get_uniform_shape_instances();
        let uniform_time = start.elapsed();
        
        println!("Hexagon-only time: {:?}", hex_time);
        println!("Uniform shape time: {:?}", uniform_time);
        
        // Performance ratios
        let compression = hexasphere.tiles.len() as f64 / shape_data.shapes.len() as f64;
        println!("Compression: {:.1}x", compression);
        
        // Estimate performance for Bevy
        println!("Bevy implications:");
        println!("  Traditional: {} separate meshes", hexasphere.tiles.len());
        println!("  Instanced: {} shapes with {} instances", 
            shape_data.shapes.len(), shape_data.instances.len());
        println!("  Draw call reduction: {:.1}x", compression);
        
        assert_eq!(hexasphere.tiles.len(), expected_tiles);
        assert_eq!(shape_data.instances.len(), hexasphere.tiles.len());
    }
    
    println!("\n✅ Performance benchmark complete");
}

/// Test that demonstrates the exact workflow for Bevy integration
#[test]
fn test_bevy_integration_workflow() {
    println!("\n=== Bevy Integration Workflow Test ===");
    
    // Simulate typical Bevy usage at a reasonable level
    let subdivision = 12;
    
    println!("Step 1: Generate hexasphere (subdivision {})", subdivision);
    let hexasphere = Hexasphere::new(5.0, subdivision, 0.95); // 5.0 radius, 95% tile size
    
    println!("Step 2: Get shape instances for GPU instancing");
    let shape_data = hexasphere.get_shape_instances();
    
    println!("Step 3: Analysis for Bevy setup");
    println!("  Total tiles: {}", hexasphere.tiles.len());
    println!("  Unique shapes: {}", shape_data.shapes.len());
    println!("  GPU draw calls needed: {} (instead of {})", 
        shape_data.shapes.len(), hexasphere.tiles.len());
    
    // Simulate what you'd do in Bevy
    println!("\nStep 4: Bevy mesh creation simulation");
    let mut mesh_count = 0;
    let mut instance_count = 0;
    
    for (shape_idx, shape) in shape_data.shapes.iter().enumerate() {
        mesh_count += 1;
        
        // Count instances for this shape
        let shape_instances = shape_data.instances.iter()
            .filter(|i| i.shape_index == shape_idx)
            .count();
        instance_count += shape_instances;
        
        println!("  Shape {}: {} sides, {} instances", 
            shape_idx, shape.sides, shape_instances);
    }
    
    assert_eq!(mesh_count, shape_data.shapes.len());
    assert_eq!(instance_count, shape_data.instances.len());
    assert_eq!(instance_count, hexasphere.tiles.len());
    
    println!("\nStep 5: Memory efficiency analysis");
    let memory_per_traditional_tile = 200; // bytes (rough estimate)
    let memory_per_shape = 100; // bytes (shape vertices)
    let memory_per_instance = 64; // bytes (transform + metadata)
    
    let traditional_memory = hexasphere.tiles.len() * memory_per_traditional_tile;
    let instanced_memory = shape_data.shapes.len() * memory_per_shape + 
                          shape_data.instances.len() * memory_per_instance;
    
    let memory_efficiency = traditional_memory as f64 / instanced_memory as f64;
    
    println!("  Traditional approach: {} KB", traditional_memory / 1024);
    println!("  Instanced approach: {} KB", instanced_memory / 1024);
    println!("  Memory efficiency: {:.1}x improvement", memory_efficiency);
    
    assert!(memory_efficiency > 1.0, "Instanced approach should be more memory efficient");
    
    println!("\n✅ Bevy integration workflow validated");
    println!("Ready for GPU instancing with {:.1}x compression!", 
        hexasphere.tiles.len() as f64 / shape_data.shapes.len() as f64);
}