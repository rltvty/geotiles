use geotiles::Hexasphere;

fn main() {
    println!("Verifying actual tile counts:");
    println!("Level | Actual | Doc Formula (10×4^(n-1)) | Your Data");
    println!("------|--------|--------------------------|----------");
    
    for level in 0..=30 {
        let hs = Hexasphere::new(1.0, level, 1.0);
        let actual = hs.tiles.len();
        
        let doc_formula = if level == 0 { 12 } else { 10 * 4_usize.pow((level - 1) as u32) };
        
        let your_data = match level {
            20 => Some(4002),
            30 => Some(9002),
            _ => None,
        };
        
        print!("{:5} | {:6} | {:24}", level, actual, doc_formula);
        if let Some(data) = your_data {
            println!(" | {:8} ✓", data);
        } else {
            println!(" |");
        }
        
        // Stop early if formula grows too large
        if doc_formula > 1_000_000 && level < 20 {
            println!("(Doc formula grows too fast, skipping to your test levels...)");
            break;
        }
    }
    
    // Try to find the real pattern
    println!("\nLooking for the real pattern:");
    let mut prev_count = 0;
    for level in 0..=10 {
        let hs = Hexasphere::new(1.0, level, 1.0);
        let actual = hs.tiles.len();
        let diff = if level > 0 { actual - prev_count } else { 0 };
        
        println!("Level {}: {} tiles (diff: {})", level, actual, diff);
        prev_count = actual;
    }
}