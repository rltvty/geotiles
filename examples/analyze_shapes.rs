use geotiles::hexasphere::{Hexasphere, ShapeAnalyzer};

fn main() {
    println!("Analyzing hexagon shape patterns at different subdivision levels:\n");
    
    for subdivisions in 2..=6 {
        let hs = Hexasphere::new(1.0, subdivisions, 1.0);
        let tiles = &hs.tiles;
        let centers: Vec<_> = tiles.iter().map(|t| t.center_point.clone()).collect();
        
        let mut analyzer = ShapeAnalyzer::new();
        analyzer.analyze_tiles(tiles, &centers);
        
        let unique_count = analyzer.unique_shape_count();
        let total_hexagons = tiles.iter().filter(|t| t.boundary.len() == 6).count();
        let total_pentagons = tiles.iter().filter(|t| t.boundary.len() == 5).count();
        
        println!("Subdivision level {}: ", subdivisions);
        println!("  Total tiles: {}", tiles.len());
        println!("  Hexagons: {}", total_hexagons);
        println!("  Pentagons: {}", total_pentagons);
        println!("  Unique hexagon shapes: {}", unique_count);
        println!("  Shape efficiency: {:.1}x reduction", tiles.len() as f64 / unique_count as f64);
        
        // Show top 3 most common shapes
        let distribution = analyzer.get_shape_distribution();
        println!("  Top 3 shapes:");
        for (i, (_, count)) in distribution.iter().take(3).enumerate() {
            println!("    Shape {}: {} tiles ({:.1}%)", 
                i + 1, count, 100.0 * *count as f64 / tiles.len() as f64);
        }
        println!();
    }
}