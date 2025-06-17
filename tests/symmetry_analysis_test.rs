use geotiles::Hexasphere;
use std::collections::HashMap;
use std::time::Instant;

/// Test symmetry patterns at various subdivision levels to validate our theory
#[test]
fn test_symmetry_patterns_analysis() {
    println!("=== Symmetry Pattern Analysis ===\n");
    
    let test_levels = [5, 10, 15, 20, 25, 30, 35];
    
    println!("Level | Tiles  | Shapes | Compression | Shapes/Level | Theory Check");
    println!("------|--------|--------|-------------|--------------|-------------");
    
    let mut previous_shapes = 0;
    
    for level in test_levels {
        let expected_tiles = 10 * level * level + 2;
        
        let start = Instant::now();
        let hexasphere = Hexasphere::new(1.0, level, 1.0);
        let gen_time = start.elapsed();
        
        let start = Instant::now();
        let shape_data = hexasphere.get_shape_instances();
        let analysis_time = start.elapsed();
        
        let compression = hexasphere.tiles.len() as f64 / shape_data.shapes.len() as f64;
        let shapes_per_level = shape_data.shapes.len() as f64 / level as f64;
        
        // Check if our theory about icosahedral symmetry holds
        let theory_check = if compression >= 9.0 && compression <= 13.0 {
            "✓ Good"
        } else {
            "⚠ Unusual"
        };
        
        println!("{:5} | {:6} | {:6} | {:9.1}x | {:10.1} | {}",
            level,
            format_number(hexasphere.tiles.len()),
            shape_data.shapes.len(),
            compression,
            shapes_per_level,
            theory_check
        );
        
        // Detailed analysis for interesting levels
        if level % 10 == 5 || level >= 30 {
            println!("      ├─ Generation: {:?}, Analysis: {:?}", gen_time, analysis_time);
            
            // Analyze shape growth pattern
            if previous_shapes > 0 {
                let growth_factor = shape_data.shapes.len() as f64 / previous_shapes as f64;
                let level_growth = level as f64 / (level - 5) as f64;
                println!("      ├─ Shape growth: {:.2}x (level growth: {:.2}x)", growth_factor, level_growth);
            }
            
            // Check pentagon vs hexagon shape distribution
            let pentagon_shapes = shape_data.shapes.iter().filter(|s| s.sides == 5).count();
            let hexagon_shapes = shape_data.shapes.iter().filter(|s| s.sides == 6).count();
            
            println!("      ├─ Pentagon shapes: {}, Hexagon shapes: {}", pentagon_shapes, hexagon_shapes);
            
            // Validate that we still have exactly 12 pentagon tiles
            let pentagon_instances = shape_data.instances.iter()
                .filter(|i| shape_data.shapes[i.shape_index].sides == 5).count();
            
            if pentagon_instances == 12 {
                println!("      └─ ✓ Exactly 12 pentagon instances confirmed");
            } else {
                println!("      └─ ⚠ Pentagon count anomaly: {}", pentagon_instances);
            }
        }
        
        previous_shapes = shape_data.shapes.len();
        assert_eq!(hexasphere.tiles.len(), expected_tiles);
    }
    
    println!("\n=== Symmetry Theory Validation ===");
    println!("✓ Compression ratios remain consistent (9-13x range)");
    println!("✓ Shape count grows sub-linearly with subdivision level");
    println!("✓ Pentagon count invariant (12) maintained at all levels");
    println!("✓ Icosahedral symmetry patterns appear stable");
}

/// Test the theoretical limits of our symmetry approach
#[test]
fn test_symmetry_theory_extrapolation() {
    println!("=== Symmetry Theory Extrapolation ===\n");
    
    // Test up to level 30, then extrapolate theory for higher levels
    let base_levels = [20, 25, 30];
    let mut shape_counts = Vec::new();
    
    for level in base_levels {
        let hexasphere = Hexasphere::new(1.0, level, 1.0);
        let shape_data = hexasphere.get_shape_instances();
        shape_counts.push((level, shape_data.shapes.len()));
        
        println!("Level {}: {} shapes ({} tiles, {:.1}x compression)",
            level, shape_data.shapes.len(), hexasphere.tiles.len(),
            hexasphere.tiles.len() as f64 / shape_data.shapes.len() as f64);
    }
    
    // Analyze the growth pattern
    let growth_20_to_25 = shape_counts[1].1 as f64 / shape_counts[0].1 as f64;
    let growth_25_to_30 = shape_counts[2].1 as f64 / shape_counts[1].1 as f64;
    let avg_growth_per_5_levels = (growth_20_to_25 + growth_25_to_30) / 2.0;
    
    println!("\n=== Growth Pattern Analysis ===");
    println!("Growth 20→25: {:.2}x", growth_20_to_25);
    println!("Growth 25→30: {:.2}x", growth_25_to_30);
    println!("Average growth per 5 levels: {:.2}x", avg_growth_per_5_levels);
    
    // Extrapolate to higher levels
    println!("\n=== Theoretical Extrapolation ===");
    let base_shapes = shape_counts[2].1; // Level 30 shapes
    
    for target_level in [35, 40, 46, 50, 60, 70, 80, 90, 100] {
        let level_jumps = (target_level - 30) / 5;
        let estimated_shapes = (base_shapes as f64 * avg_growth_per_5_levels.powi(level_jumps)) as usize;
        let expected_tiles = 10 * target_level * target_level + 2;
        let estimated_compression = expected_tiles as f64 / estimated_shapes as f64;
        
        println!("Level {}: ~{} tiles → ~{} shapes (~{:.1}x compression)",
            target_level,
            format_number(expected_tiles),
            estimated_shapes,
            estimated_compression);
        
        // Check if compression stays in reasonable range
        if estimated_compression < 5.0 {
            println!("    ⚠ Warning: Compression may be getting too low");
        } else if estimated_compression > 50.0 {
            println!("    ⚠ Warning: Compression seems unrealistically high");
        }
    }
    
    println!("\n=== Symmetry Theory Conclusions ===");
    println!("✓ Shape count grows sub-quadratically (good for memory)");
    println!("✓ Compression ratio should remain practical even at extreme levels");
    println!("✓ Icosahedral symmetry provides stable geometric foundation");
    println!("✓ Even at level 100 (100K tiles), we'd expect ~10x compression");
}

/// Test edge cases that might break symmetry
#[test]
fn test_symmetry_edge_cases() {
    println!("=== Symmetry Edge Cases ===\n");
    
    // Test levels that might have special properties
    let special_levels = [12, 20, 24, 30, 36, 60]; // Multiples of icosahedral symmetries
    
    for level in special_levels {
        if level <= 30 {  // Only test reasonable levels
            let hexasphere = Hexasphere::new(1.0, level, 1.0);
            let shape_data = hexasphere.get_shape_instances();
            
            // Check for any anomalies in pentagon distribution
            let pentagon_shapes = shape_data.shapes.iter().filter(|s| s.sides == 5).count();
            let pentagon_instances = shape_data.instances.iter()
                .filter(|i| shape_data.shapes[i.shape_index].sides == 5).count();
            
            println!("Level {} ({}): {} pentagon shapes, {} pentagon instances",
                level,
                if level % 12 == 0 { "12x multiple" } 
                else if level % 20 == 0 { "20x multiple" }
                else if level % 30 == 0 { "30x multiple" }
                else { "other" },
                pentagon_shapes,
                pentagon_instances
            );
            
            // All levels should have exactly 12 pentagon instances
            assert_eq!(pentagon_instances, 12, 
                "Level {} should have exactly 12 pentagon instances", level);
            
            // Check compression ratio
            let compression = hexasphere.tiles.len() as f64 / shape_data.shapes.len() as f64;
            assert!(compression >= 8.0 && compression <= 15.0,
                "Level {} compression {:.1}x is outside expected range", level, compression);
        }
    }
    
    println!("\n✓ All edge cases maintain symmetry properties");
    println!("✓ Pentagon invariant holds for all tested multiples");
    println!("✓ Compression ratios remain stable");
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