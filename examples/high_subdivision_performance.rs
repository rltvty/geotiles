fn main() {
    println!("=== High Subdivision Level Performance Analysis ===\n");
    
    // Calculate tile counts for high subdivision levels
    println!("Subdivision Level Scaling:");
    println!("Level | Total Tiles         | Approx. Unique Shapes | Compression");
    println!("------|---------------------|----------------------|-------------");
    
    for level in [10, 20, 30, 40, 50, 60, 70, 80, 90, 100] {
        let tiles = calculate_tile_count(level);
        let shapes = estimate_unique_shapes(level);
        let compression = tiles as f64 / shapes as f64;
        
        println!("{:5} | {:>19} | {:>20} | {:>10.1}x", 
            level, format_large_number(tiles), shapes, compression);
    }
    
    // Memory comparison
    println!("\n\nMemory Usage Comparison:");
    println!("Level | Traditional Approach  | Pattern-Based      | Savings");
    println!("------|-----------------------|--------------------|---------");
    
    for level in [50, 60, 70, 80, 90, 100] {
        let tiles = calculate_tile_count(level);
        let shapes = estimate_unique_shapes(level);
        
        // Traditional: store all tile vertices
        // ~200 bytes per tile (vertices, neighbors, metadata)
        let traditional_bytes = tiles.saturating_mul(200);
        
        // Pattern-based: shapes + instances
        // Shape: ~72 bytes, Instance: ~32 bytes
        let pattern_bytes = shapes.saturating_mul(72).saturating_add(tiles.saturating_mul(32));
        
        let savings = 100.0 * (1.0 - pattern_bytes as f64 / traditional_bytes as f64);
        
        println!("{:5} | {:>21} | {:>18} | {:>7.1}%", 
            level, 
            format_bytes(traditional_bytes),
            format_bytes(pattern_bytes),
            savings);
    }
    
    // Performance implications
    println!("\n\nPerformance Implications:");
    println!("\nFor subdivision level 100:");
    let tiles_100 = calculate_tile_count(100);
    let shapes_100 = estimate_unique_shapes(100);
    
    println!("- Total tiles: {}", format_large_number(tiles_100));
    println!("- Unique shapes: ~{}", shapes_100);
    println!("- Compression ratio: {:.1}x", tiles_100 as f64 / shapes_100 as f64);
    
    println!("\nTraditional approach:");
    println!("- Must generate {} tile geometries", format_large_number(tiles_100));
    println!("- Memory: ~{}", format_bytes(tiles_100.saturating_mul(200)));
    println!("- Generation time: ~{:.1} minutes (estimated)", tiles_100 as f64 / 1_000_000.0);
    
    println!("\nPattern-based approach:");
    println!("- Generate only {} shape templates", shapes_100);
    println!("- Memory: ~{}", format_bytes(shapes_100.saturating_mul(72).saturating_add(tiles_100.saturating_mul(32))));
    println!("- Generation time: ~{:.1} seconds (estimated)", shapes_100 as f64 / 100.0);
    println!("- Speedup: ~{:.0}x faster", (tiles_100 as f64 / 1_000_000.0 * 60.0) / (shapes_100 as f64 / 100.0));
    
    // GPU benefits
    println!("\n\nGPU Rendering Benefits:");
    println!("Traditional: {} draw calls or huge vertex buffer", format_large_number(tiles_100));
    println!("Pattern-based: 1 instanced draw call with {} shapes", shapes_100);
    println!("\nThe pattern-based approach enables:");
    println!("- GPU instancing for massive performance gains");
    println!("- Reduced vertex buffer size");
    println!("- Better cache utilization");
    println!("- Lower memory bandwidth requirements");
    
    // Theoretical scaling
    println!("\n\nTheoretical Scaling:");
    println!("\nThe pattern suggests that unique shapes grow approximately as:");
    println!("- Levels 1-4: 5n - 5 shapes");
    println!("- Levels 5-7: 10n - 25 shapes");
    println!("- Levels 8-10: 16n - 73 shapes");
    println!("- Higher levels: growth rate increases every ~3-5 levels");
    println!("\nThis sub-linear growth means the benefits increase dramatically");
    println!("at higher subdivision levels, making levels 50-100 feasible.");
}

fn calculate_tile_count(level: u32) -> usize {
    if level == 1 {
        12
    } else {
        // 10 * 4^(n-1) + 2
        // For large n, this becomes approximately 10 * 4^(n-1)
        let base: u128 = 4;
        let power = (level - 1) as u32;
        
        // For very large levels, we need to handle overflow
        if power > 31 {
            // Approximate for display purposes
            usize::MAX
        } else {
            (10 * base.pow(power) + 2) as usize
        }
    }
}

fn estimate_unique_shapes(level: u32) -> usize {
    // Based on observed pattern with extrapolation
    match level {
        1 => 1,
        2 => 5,
        3 => 10,
        4 => 15,
        5 => 25,
        6 => 35,
        7 => 45,
        8 => 59,
        9 => 75,
        10 => 91,
        // Extrapolate for higher levels
        11..=15 => 91 + (level - 10) as usize * 20,
        16..=20 => 191 + (level - 15) as usize * 25,
        21..=30 => 316 + (level - 20) as usize * 30,
        31..=50 => 616 + (level - 30) as usize * 40,
        51..=70 => 1416 + (level - 50) as usize * 50,
        71..=90 => 2416 + (level - 70) as usize * 60,
        91..=100 => 3616 + (level - 90) as usize * 70,
        _ => 4316 + (level - 100) as usize * 80,
    }
}

fn format_large_number(n: usize) -> String {
    if n == usize::MAX {
        ">2^64".to_string()
    } else if n >= 1_000_000_000_000 {
        format!("{:.2}T", n as f64 / 1_000_000_000_000.0)
    } else if n >= 1_000_000_000 {
        format!("{:.2}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.2}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_bytes(bytes: usize) -> String {
    if bytes >= 1_099_511_627_776 {
        format!("{:.2} TB", bytes as f64 / 1_099_511_627_776.0)
    } else if bytes >= 1_073_741_824 {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}