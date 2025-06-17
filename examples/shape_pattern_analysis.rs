use geotiles::hexasphere::{Hexasphere, ShapeAnalyzer};
use std::collections::HashMap;

fn main() {
    println!("=== Analyzing Shape Patterns in Geodesic Polyhedra ===\n");
    
    // Store results for pattern analysis
    let mut results = Vec::new();
    
    // Test a wider range of subdivision levels
    for subdivisions in 1..=8 {
        println!("Subdivision level {}: ", subdivisions);
        let hs = Hexasphere::new(1.0, subdivisions, 1.0);
        let tiles = &hs.tiles;
        let centers: Vec<_> = tiles.iter().map(|t| t.center_point.clone()).collect();
        
        let mut analyzer = ShapeAnalyzer::new();
        analyzer.analyze_tiles(tiles, &centers);
        
        let unique_count = analyzer.unique_shape_count();
        let total_tiles = tiles.len();
        let hexagons = tiles.iter().filter(|t| t.boundary.len() == 6).count();
        let pentagons = tiles.iter().filter(|t| t.boundary.len() == 5).count();
        
        println!("  Total tiles: {}", total_tiles);
        println!("  Hexagons: {}, Pentagons: {}", hexagons, pentagons);
        println!("  Unique shapes: {}", unique_count);
        
        // Get detailed shape distribution
        let distribution = analyzer.get_shape_distribution();
        let shape_indices = analyzer.get_shape_indices();
        
        // Count pentagon shapes vs hexagon shapes
        let mut pentagon_shapes = 0;
        let mut hexagon_shapes = 0;
        
        for (i, (_, indices)) in distribution.iter().enumerate() {
            // Check if this shape is a pentagon or hexagon by looking at first instance
            let first_tile_idx = shape_indices.iter().position(|&idx| idx == i).unwrap();
            if tiles[first_tile_idx].boundary.len() == 5 {
                pentagon_shapes += 1;
            } else {
                hexagon_shapes += 1;
            }
        }
        
        println!("  Pentagon shapes: {}, Hexagon shapes: {}", pentagon_shapes, hexagon_shapes);
        
        // Show shape frequency distribution
        let mut freq_map: HashMap<usize, usize> = HashMap::new();
        for (_, count) in &distribution {
            *freq_map.entry(*count).or_insert(0) += 1;
        }
        
        print!("  Shape frequencies: ");
        let mut freq_vec: Vec<_> = freq_map.iter().collect();
        freq_vec.sort_by_key(|&(count, _)| count);
        for (count, num_shapes) in freq_vec {
            print!("{} shapes appear {} times, ", num_shapes, count);
        }
        println!();
        
        results.push((subdivisions, unique_count, hexagon_shapes, pentagon_shapes));
        println!();
    }
    
    // Analyze patterns
    println!("=== Pattern Analysis ===\n");
    
    println!("Level | Total | Hex Shapes | Pent Shapes | Formula Check");
    println!("------|-------|------------|-------------|---------------");
    
    for (level, total, hex_shapes, pent_shapes) in &results {
        // Check various formulas
        let formula1 = 5 * (level - 1); // Works for levels 2-4
        let formula2 = 5 * level; // Works for level 5?
        let formula3 = 5 * level - 5; // Same as formula1
        
        // Check if it follows a recursive pattern
        let expected_tiles = if *level == 1 { 12 } else { 10 * 4_usize.pow((level - 1) as u32) + 2 };
        
        println!("{:5} | {:5} | {:10} | {:11} | 5*(n-1)={:3}, 5*n={:3}", 
            level, total, hex_shapes, pent_shapes, formula1, formula2);
    }
    
    // Check differences between levels
    println!("\n=== Growth Pattern ===\n");
    println!("Transition | Shape Increase | Tile Increase | Ratio");
    println!("-----------|----------------|---------------|-------");
    
    for i in 1..results.len() {
        let (prev_level, prev_shapes, _, _) = results[i-1];
        let (curr_level, curr_shapes, _, _) = results[i];
        
        let prev_tiles = if prev_level == 1 { 12 } else { 10 * 4_usize.pow((prev_level - 1) as u32) + 2 };
        let curr_tiles = if curr_level == 1 { 12 } else { 10 * 4_usize.pow((curr_level - 1) as u32) + 2 };
        
        let shape_increase = curr_shapes - prev_shapes;
        let tile_increase = curr_tiles - prev_tiles;
        
        println!("{} -> {}     | {:14} | {:13} | {:.3}", 
            prev_level, curr_level, shape_increase, tile_increase, 
            tile_increase as f64 / shape_increase as f64);
    }
    
    // Theoretical analysis
    println!("\n=== Theoretical Considerations ===\n");
    
    println!("In geodesic polyhedra, tiles at similar 'distances' from pentagons");
    println!("tend to have similar shapes. The distance is measured in terms of");
    println!("the graph distance on the dual polyhedron.");
    println!();
    println!("Key observations:");
    println!("1. There are always exactly 12 pentagons (from icosahedron vertices)");
    println!("2. Hexagons can be classified by their proximity to pentagons");
    println!("3. The subdivision process creates 'rings' of similar hexagons");
    println!("4. Higher subdivisions create more distinct 'distance classes'");
}