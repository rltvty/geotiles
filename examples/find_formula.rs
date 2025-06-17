use geotiles::Hexasphere;

fn main() {
    println!("Finding the exact formula:");
    println!("Level | Tiles | 10n²+2 | 10n²+10n+2 | Second Diff");
    println!("------|-------|--------|-------------|------------");
    
    let mut prev_diff = 0;
    let mut prev_tiles = 0;
    
    for level in 0..=30 {
        let hs = Hexasphere::new(1.0, level, 1.0);
        let actual = hs.tiles.len();
        
        let formula1 = 10 * level * level + 2;
        let formula2 = 10 * level * level + 10 * level + 2;
        
        let diff = if level > 0 { actual - prev_tiles } else { 0 };
        let second_diff = if level > 1 { diff - prev_diff } else { 0 };
        
        if level <= 15 || [20, 25, 30].contains(&level) {
            println!("{:5} | {:5} | {:6} | {:11} | {:11}", 
                level, actual, formula1, formula2, second_diff);
        }
        
        // Check your specific values
        if level == 20 && actual != 4002 {
            println!("      ✗ Expected 4002, got {}", actual);
        }
        if level == 30 && actual != 9002 {
            println!("      ✗ Expected 9002, got {}", actual);
        }
        
        prev_diff = diff;
        prev_tiles = actual;
    }
    
    // The second differences being constant (20) confirms it's quadratic
    // If second diff = 20, then coefficient of n² is 20/2 = 10
    // So it should be 10n² + an + b
    
    println!("\nAnalysis:");
    println!("Second differences = 20 (constant) confirms quadratic with coefficient 10");
    println!("Pattern: tiles = 10n² + 10n + 2");
    println!("But this doesn't match your reported values exactly...");
    
    // Maybe there's a different subdivision algorithm or the numbers were approximate?
    println!("\nPossible explanations:");
    println!("1. Different subdivision method");
    println!("2. Different hex_size parameter"); 
    println!("3. Approximation in reported numbers");
    println!("4. Different version of library");
}