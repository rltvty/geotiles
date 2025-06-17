use geotiles::hexasphere::{Hexasphere, ShapeAnalyzer};
use geotiles::hexasphere::shape_theory::{predict_unique_shapes, calculate_instancing_benefit, shape_distribution_theory};
use geotiles::hexasphere::direct_shapes::{generate_unique_shapes_direct, ShapePlacementRules};

fn main() {
    println!("=== Direct Shape Generation Demo ===\n");
    
    // Show the theory
    println!("{}", shape_distribution_theory());
    
    // Test predictions vs actual
    println!("\n=== Prediction Accuracy ===\n");
    println!("Level | Predicted | Actual | Error");
    println!("------|-----------|--------|-------");
    
    for level in 1..=8 {
        let predicted = predict_unique_shapes(level);
        
        // Generate actual hexasphere to compare
        let hs = Hexasphere::new(1.0, level as usize, 1.0);
        let tiles = &hs.tiles;
        let centers: Vec<_> = tiles.iter().map(|t| t.center_point.clone()).collect();
        
        let mut analyzer = ShapeAnalyzer::new();
        analyzer.analyze_tiles(tiles, &centers);
        let actual = analyzer.unique_shape_count();
        
        let error = (predicted as i32 - actual as i32).abs();
        
        println!("{:5} | {:9} | {:6} | {:5}", level, predicted, actual, error);
    }
    
    // Demonstrate instancing benefits
    println!("\n=== Instancing Benefits ===\n");
    println!("Level | Tiles  | Shapes | Benefit");
    println!("------|--------|--------|--------");
    
    for level in 1..=10 {
        let tiles = if level == 1 { 12 } else { 10 * 4_usize.pow(level - 1) + 2 };
        let shapes = predict_unique_shapes(level);
        let benefit = calculate_instancing_benefit(level);
        
        println!("{:5} | {:6} | {:6} | {:5.1}x", level, tiles, shapes, benefit);
    }
    
    // Test direct shape generation
    println!("\n=== Direct Shape Generation ===\n");
    
    for level in 2..=5 {
        println!("Subdivision level {}: ", level);
        
        let shapes = generate_unique_shapes_direct(level);
        let rules = ShapePlacementRules::new(&shapes);
        
        println!("  Generated {} unique shapes", shapes.len());
        
        // Count shape types
        let pentagons = shapes.iter().filter(|s| s.sides == 5).count();
        let hexagons = shapes.iter().filter(|s| s.sides == 6).count();
        println!("  Pentagons: {}, Hexagons: {}", pentagons, hexagons);
        
        // Show instance distribution
        let mut total_instances = 0;
        for i in 0..shapes.len() {
            let count = rules.instance_count(i);
            total_instances += count;
        }
        println!("  Total instances from rules: {}", total_instances);
        
        // Compare with actual
        let expected_tiles = if level == 1 { 12 } else { 10 * 4_usize.pow(level - 1) + 2 };
        println!("  Expected tiles: {}", expected_tiles);
        println!("  Coverage: {:.1}%", 100.0 * total_instances as f64 / expected_tiles as f64);
        println!();
    }
    
    // Explain the approach
    println!("\n=== Direct Generation Approach ===\n");
    println!("Instead of generating all tiles and analyzing them, we can:");
    println!("1. Predict the number of unique shapes using the discovered formula");
    println!("2. Generate representative shapes based on distance from pentagons");
    println!("3. Use icosahedral symmetry to determine instance positions");
    println!();
    println!("This approach could enable:");
    println!("- Instant shape generation for any subdivision level");
    println!("- Memory-efficient representation (shapes + rules only)");
    println!("- Direct computation of tile properties without full mesh");
    
    // Future work
    println!("\n=== Future Improvements ===\n");
    println!("The current direct generation is simplified. To make it production-ready:");
    println!("1. Derive exact mathematical formulas for shape variations");
    println!("2. Implement precise icosahedral coordinate generation");
    println!("3. Calculate exact edge lengths and angles from theory");
    println!("4. Validate against full mesh generation for all levels");
}