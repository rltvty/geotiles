use geotiles::hexasphere::shape_theory::{predict_unique_shapes, calculate_instancing_benefit};

fn main() {
    println!("=== High Subdivision Level Analysis (50-100) ===\n");
    
    // Memory analysis
    println!("Memory Requirements Analysis:");
    println!("Level | Total Tiles    | Traditional (GB) | Instanced (MB) | Reduction");
    println!("------|----------------|------------------|----------------|----------");
    
    for level in [10, 20, 30, 40, 50, 60, 70, 80, 90, 100] {
        let tiles = calculate_tile_count(level);
        let shapes = predict_unique_shapes(level);
        
        // Memory calculations
        // Traditional: Each tile stores ~6 vertices × 3 floats × 4 bytes = 72 bytes minimum
        // Plus neighbor data, orientation, etc. ~200 bytes per tile realistic
        let traditional_bytes = tiles * 200;
        let traditional_gb = traditional_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        
        // Instanced: Only store unique shapes + instance data
        // Shape: ~6 vertices × 3 floats × 4 bytes = 72 bytes
        // Instance: position (12) + rotation (16) + index (4) = 32 bytes
        let shape_bytes = shapes * 72;
        let instance_bytes = tiles * 32;
        let instanced_bytes = shape_bytes + instance_bytes;
        let instanced_mb = instanced_bytes as f64 / (1024.0 * 1024.0);
        
        let reduction = traditional_bytes as f64 / instanced_bytes as f64;
        
        println!("{:5} | {:14} | {:16.2} | {:14.2} | {:7.1}x", 
            level, tiles, traditional_gb, instanced_mb, reduction);
    }
    
    // Performance analysis
    println!("\n\nPerformance Benefits:");
    println!("Level | Shape Count | Instancing Benefit | Mesh Creation Speedup");
    println!("------|-------------|--------------------|-----------------------");
    
    for level in [50, 60, 70, 80, 90, 100] {
        let tiles = calculate_tile_count(level);
        let shapes = predict_unique_shapes(level);
        let benefit = calculate_instancing_benefit(level);
        
        // Mesh creation is O(shapes) instead of O(tiles)
        let speedup = tiles as f64 / shapes as f64;
        
        println!("{:5} | {:11} | {:17.1}x | {:20.1}x", 
            level, shapes, benefit, speedup);
    }
    
    // Pattern extrapolation
    println!("\n\nPattern Extrapolation for High Subdivisions:");
    println!("Based on the observed pattern, we can extrapolate:");
    println!();
    
    for level in [50, 60, 70, 80, 90, 100] {
        let shapes = predict_unique_shapes(level);
        let growth_rate = calculate_growth_rate(level);
        
        println!("Level {}: ~{} unique shapes (growth rate: {} per level)", 
            level, shapes, growth_rate);
    }
    
    // Theoretical limits
    println!("\n\nTheoretical Considerations:");
    println!();
    println!("At very high subdivision levels (50-100):");
    println!("1. The number of unique shapes grows sub-linearly");
    println!("2. Most tiles belong to the 'general position' symmetry class");
    println!("3. Edge effects near pentagons become relatively smaller");
    println!("4. The mesh approaches a nearly uniform hexagonal tiling");
    
    // Implementation strategy
    println!("\n\nImplementation Strategy for Level 50-100:");
    println!();
    println!("1. **Direct Shape Generation**:");
    println!("   - Pre-calculate shape templates mathematically");
    println!("   - No need to generate full mesh");
    println!("   - O(shapes) complexity instead of O(tiles)");
    println!();
    println!("2. **Hierarchical Representation**:");
    println!("   - Group shapes by distance from pentagons");
    println!("   - Use symmetry groups for efficient placement");
    println!("   - Lazy evaluation of tile positions");
    println!();
    println!("3. **GPU-Friendly Data**:");
    println!("   - Shape buffer: vertex data for unique shapes");
    println!("   - Instance buffer: transforms for all tiles");
    println!("   - Single draw call with instancing");
    
    // Example calculation for level 100
    println!("\n\nExample: Subdivision Level 100");
    let tiles_100 = calculate_tile_count(100);
    let shapes_100 = predict_unique_shapes(100);
    
    println!("Total tiles: {}", tiles_100);
    println!("Unique shapes: {}", shapes_100);
    println!("Compression ratio: {:.1}x", tiles_100 as f64 / shapes_100 as f64);
    println!();
    println!("Using pattern-based generation:");
    println!("- Shape generation time: ~{} ms", shapes_100 / 10); // Estimate
    println!("- Full mesh generation time: ~{} seconds", tiles_100 / 100_000); // Estimate
    println!("- Speedup: ~{:.0}x faster", (tiles_100 as f64 / 100_000.0) / (shapes_100 as f64 / 10.0 / 1000.0));
}

fn calculate_tile_count(level: u32) -> usize {
    if level == 1 {
        12
    } else {
        10 * 4_usize.pow(level - 1) + 2
    }
}

fn calculate_growth_rate(level: u32) -> u32 {
    // Based on observed pattern: growth rate increases every ~3 levels
    match level {
        1..=4 => 5,
        5..=7 => 10,
        8..=10 => 16,
        11..=15 => 20,
        16..=20 => 25,
        21..=30 => 30,
        31..=50 => 40,
        51..=70 => 50,
        71..=90 => 60,
        _ => 70,
    }
}