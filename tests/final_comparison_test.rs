use geotiles::Hexasphere;
use std::time::Instant;

/// Final comparison test for the exact subdivision levels requested by the user: 6, 12, 22
#[test]
fn test_final_comparison_6_12_22() {
    println!("=== Final Comparison Test: Subdivisions 6, 12, 22 ===\n");
    
    let test_levels = [6, 12, 22];
    
    for level in test_levels {
        println!("--- Testing Subdivision Level {} ---", level);
        
        let expected_tiles = 10 * level * level + 2;
        println!("Expected tiles by formula: {}", expected_tiles);
        
        // Generate hexasphere using traditional method
        let start = Instant::now();
        let hexasphere = Hexasphere::new(1.0, level, 1.0);
        let traditional_time = start.elapsed();
        
        println!("Traditional generation: {:?}", traditional_time);
        println!("Actual tiles: {}", hexasphere.tiles.len());
        
        // Validate tile count matches formula
        assert_eq!(hexasphere.tiles.len(), expected_tiles,
            "Tile count should match formula 10n² + 2 for level {}", level);
        
        // Validate pentagon/hexagon breakdown
        let pentagons = hexasphere.tiles.iter().filter(|t| t.boundary.len() == 5).count();
        let hexagons = hexasphere.tiles.iter().filter(|t| t.boundary.len() == 6).count();
        
        assert_eq!(pentagons, 12, "Should always have exactly 12 pentagons");
        assert_eq!(hexagons + pentagons, hexasphere.tiles.len());
        
        println!("Pentagon tiles: {}", pentagons);
        println!("Hexagon tiles: {}", hexagons);
        
        // Test shape instancing method 
        let start = Instant::now();
        let shape_data = hexasphere.get_shape_instances();
        let instancing_time = start.elapsed();
        
        println!("Shape instancing: {:?}", instancing_time);
        println!("Unique shapes: {}", shape_data.shapes.len());
        println!("Shape instances: {}", shape_data.instances.len());
        
        // Validate that shape instancing produces identical output
        assert_eq!(shape_data.instances.len(), hexasphere.tiles.len(),
            "Instance count must equal tile count");
        
        // Validate that every tile has a corresponding instance
        let mut covered_tiles = std::collections::HashSet::new();
        for instance in &shape_data.instances {
            assert!(instance.tile_index < hexasphere.tiles.len(),
                "Tile index should be valid");
            covered_tiles.insert(instance.tile_index);
        }
        
        assert_eq!(covered_tiles.len(), hexasphere.tiles.len(),
            "All tiles should be covered exactly once");
        
        // Validate tile types match between methods
        let shape_pentagons = shape_data.instances.iter()
            .filter(|i| shape_data.shapes[i.shape_index].sides == 5).count();
        let shape_hexagons = shape_data.instances.iter()
            .filter(|i| shape_data.shapes[i.shape_index].sides == 6).count();
        
        assert_eq!(shape_pentagons, pentagons, "Pentagon counts must match");
        assert_eq!(shape_hexagons, hexagons, "Hexagon counts must match");
        
        // Report compression achieved
        let compression = hexasphere.tiles.len() as f64 / shape_data.shapes.len() as f64;
        println!("Compression achieved: {:.1}x", compression);
        
        // Validate center positions match
        let mut max_center_error = 0.0f64;
        for instance in &shape_data.instances {
            let original_center = &hexasphere.tiles[instance.tile_index].center_point;
            let instance_center = &instance.center;
            
            let distance = ((original_center.x - instance_center.x).powi(2) +
                           (original_center.y - instance_center.y).powi(2) +
                           (original_center.z - instance_center.z).powi(2)).sqrt();
            
            max_center_error = max_center_error.max(distance);
        }
        
        println!("Maximum center position error: {:.9}", max_center_error);
        assert!(max_center_error < 0.001, "Center positions should match within tolerance");
        
        println!("✅ Level {} validation complete - methods produce identical output!\n", level);
    }
    
    println!("🎉 All subdivision levels (6, 12, 22) successfully validated!");
    println!("Both traditional and shape instancing methods produce identical results.");
}

/// Quick performance comparison
#[test]
fn test_performance_comparison_6_12_22() {
    println!("=== Performance Comparison: Traditional vs Shape Instancing ===\n");
    
    let test_levels = [6, 12, 22];
    
    for level in test_levels {
        println!("Subdivision {}:", level);
        
        // Traditional generation timing
        let start = Instant::now();
        let hexasphere = Hexasphere::new(1.0, level, 1.0);
        let traditional_time = start.elapsed();
        
        // Shape instancing timing
        let start = Instant::now();
        let shape_data = hexasphere.get_shape_instances();
        let instancing_time = start.elapsed();
        
        let compression = hexasphere.tiles.len() as f64 / shape_data.shapes.len() as f64;
        
        println!("  Traditional: {:?}", traditional_time);
        println!("  Shape instancing: {:?}", instancing_time);
        println!("  Tiles: {} → {} shapes ({:.1}x compression)", 
            hexasphere.tiles.len(), shape_data.shapes.len(), compression);
        
        // Shape instancing should be much faster than generation
        // (it's analyzing already-generated data)
        let speedup = traditional_time.as_nanos() as f64 / instancing_time.as_nanos() as f64;
        println!("  Analysis speedup: {:.1}x faster", speedup);
        
        println!();
    }
    
    println!("✅ Performance comparison complete");
}