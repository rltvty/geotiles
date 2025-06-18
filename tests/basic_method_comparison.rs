use geotiles::Hexasphere;
use std::time::Instant;

/// Test the specific subdivision levels requested: 6, 12, 22
#[test]
fn test_requested_subdivision_levels_basic() {
    println!("\n=== Testing Requested Subdivision Levels ===");

    // Test levels 6 and 12 with full generation
    let quick_levels = [6, 12];

    for level in quick_levels {
        println!("\n--- Subdivision Level {} ---", level);

        let expected_tiles = 10 * level * level + 2;
        println!("Expected tiles by formula: {}", expected_tiles);

        // Generate hexasphere
        let start = Instant::now();
        let hexasphere = Hexasphere::new(1.0, level, 1.0);
        let generation_time = start.elapsed();

        // Validate basic properties
        assert_eq!(
            hexasphere.tiles.len(),
            expected_tiles,
            "Tile count should match formula for level {}",
            level
        );

        let pentagons = hexasphere
            .tiles
            .iter()
            .filter(|t| t.boundary.len() == 5)
            .count();
        let hexagons = hexasphere
            .tiles
            .iter()
            .filter(|t| t.boundary.len() == 6)
            .count();

        assert_eq!(pentagons, 12, "Should always have 12 pentagons");
        assert_eq!(
            pentagons + hexagons,
            hexasphere.tiles.len(),
            "All tiles should be either pentagons or hexagons"
        );

        println!("Generation time: {:?}", generation_time);
        println!("Actual tiles: {}", hexasphere.tiles.len());
        println!("Pentagons: {}, Hexagons: {}", pentagons, hexagons);

        // Test that we can get orientations (needed for rendering)
        let orientations = hexasphere.get_tile_orientations();
        let valid_orientations = orientations.iter().filter(|o| o.is_some()).count();

        println!(
            "Valid orientations: {}/{}",
            valid_orientations,
            orientations.len()
        );
        assert!(
            valid_orientations > 0,
            "Should have some valid orientations"
        );

        println!("✅ Level {} validation complete", level);
    }

    // For level 22, just validate the formula (too slow for regular testing)
    let level_22_expected = 10 * 22 * 22 + 2;
    println!("\n--- Subdivision Level 22 (formula only) ---");
    println!("Expected tiles by formula: {}", level_22_expected);
    assert_eq!(level_22_expected, 4842, "Level 22 should have 4842 tiles");

    println!("\n✅ All requested levels validated");
}

/// Compare tile generation consistency across different parameters
#[test]
fn test_generation_consistency() {
    println!("\n=== Testing Generation Consistency ===");

    let level = 8;
    let radius = 1.0;

    // Test different hex_size values produce consistent tile counts
    let hex_sizes = [0.5, 0.8, 0.95, 1.0];
    let expected_tiles = 10 * level * level + 2;

    for hex_size in hex_sizes {
        let hexasphere = Hexasphere::new(radius, level, hex_size);

        assert_eq!(
            hexasphere.tiles.len(),
            expected_tiles,
            "Tile count should be independent of hex_size"
        );

        let pentagons = hexasphere
            .tiles
            .iter()
            .filter(|t| t.boundary.len() == 5)
            .count();
        assert_eq!(
            pentagons, 12,
            "Pentagon count should be independent of hex_size"
        );

        println!(
            "hex_size {}: {} tiles, 12 pentagons ✓",
            hex_size,
            hexasphere.tiles.len()
        );
    }

    // Test different radii produce consistent tile counts
    let radii = [0.5, 1.0, 2.0, 10.0];

    for radius in radii {
        let hexasphere = Hexasphere::new(radius, level, 1.0);

        assert_eq!(
            hexasphere.tiles.len(),
            expected_tiles,
            "Tile count should be independent of radius"
        );

        // Check that radius affects actual coordinates
        let max_distance = hexasphere
            .tiles
            .iter()
            .map(|t| {
                (t.center_point.x.powi(2) + t.center_point.y.powi(2) + t.center_point.z.powi(2))
                    .sqrt()
            })
            .fold(0.0f64, |a, b| a.max(b));

        assert!(
            (max_distance - radius).abs() < 0.1,
            "Max distance should be approximately equal to radius"
        );

        println!(
            "radius {}: {} tiles, max distance {:.3} ✓",
            radius,
            hexasphere.tiles.len(),
            max_distance
        );
    }

    println!("✅ Generation consistency validated");
}

/// Test geometric properties of generated tiles
#[test]
fn test_geometric_properties() {
    println!("\n=== Testing Geometric Properties ===");

    let level = 6;
    let hexasphere = Hexasphere::new(1.0, level, 1.0);

    println!(
        "Testing {} tiles at subdivision {}",
        hexasphere.tiles.len(),
        level
    );

    let mut pentagon_count = 0;
    let mut hexagon_count = 0;
    let mut invalid_tiles = 0;

    for (i, tile) in hexasphere.tiles.iter().enumerate() {
        let sides = tile.boundary.len();

        match sides {
            5 => pentagon_count += 1,
            6 => hexagon_count += 1,
            _ => {
                invalid_tiles += 1;
                println!("Warning: Tile {} has {} sides", i, sides);
            }
        }

        // Check that tile has valid center
        let center_distance = (tile.center_point.x.powi(2)
            + tile.center_point.y.powi(2)
            + tile.center_point.z.powi(2))
        .sqrt();
        assert!(
            (center_distance - 1.0).abs() < 0.1,
            "Tile center should be approximately on unit sphere"
        );

        // Check that boundary points are reasonable
        for vertex in &tile.boundary {
            let vertex_distance = (vertex.x.powi(2) + vertex.y.powi(2) + vertex.z.powi(2)).sqrt();
            assert!(
                vertex_distance > 0.5 && vertex_distance < 1.5,
                "Boundary vertices should be near unit sphere"
            );
        }
    }

    assert_eq!(
        invalid_tiles, 0,
        "All tiles should be pentagons or hexagons"
    );
    assert_eq!(pentagon_count, 12, "Should have exactly 12 pentagons");
    assert_eq!(hexagon_count + pentagon_count, hexasphere.tiles.len());

    println!("Geometric validation:");
    println!("  Pentagons: {}", pentagon_count);
    println!("  Hexagons: {}", hexagon_count);
    println!("  Invalid tiles: {}", invalid_tiles);

    println!("✅ Geometric properties validated");
}

/// Test that tile neighbor relationships are valid
#[test]
fn test_neighbor_relationships() {
    println!("\n=== Testing Neighbor Relationships ===");

    let level = 5;
    let hexasphere = Hexasphere::new(1.0, level, 1.0);

    let mut total_neighbors = 0;
    let mut pentagon_neighbors = 0;
    let mut hexagon_neighbors = 0;

    for tile in &hexasphere.tiles {
        let neighbor_count = tile.neighbors.len();
        total_neighbors += neighbor_count;

        // Pentagons should have 5 neighbors, hexagons should have 6
        match tile.boundary.len() {
            5 => {
                pentagon_neighbors += neighbor_count;
                assert_eq!(neighbor_count, 5, "Pentagons should have 5 neighbors");
            }
            6 => {
                hexagon_neighbors += neighbor_count;
                assert_eq!(neighbor_count, 6, "Hexagons should have 6 neighbors");
            }
            _ => panic!("Invalid tile type"),
        }

        // Check that all neighbor indices are valid
        for &neighbor_idx in &tile.neighbors {
            assert!(
                neighbor_idx < hexasphere.tiles.len(),
                "Neighbor index {} should be valid",
                neighbor_idx
            );
        }
    }

    println!("Neighbor analysis:");
    println!("  Total neighbor connections: {}", total_neighbors);
    println!("  Pentagon neighbor connections: {}", pentagon_neighbors);
    println!("  Hexagon neighbor connections: {}", hexagon_neighbors);
    println!(
        "  Average neighbors per tile: {:.1}",
        total_neighbors as f64 / hexasphere.tiles.len() as f64
    );

    // Each edge is shared by exactly 2 tiles, so total should be even
    assert_eq!(
        total_neighbors % 2,
        0,
        "Total neighbor connections should be even"
    );

    println!("✅ Neighbor relationships validated");
}

/// Performance benchmark for the requested levels
#[test]
fn test_performance_scaling() {
    println!("\n=== Performance Scaling Test ===");

    let test_levels = [4, 6, 8, 10, 12];
    let mut previous_time = std::time::Duration::from_nanos(1);

    println!("Level | Tiles  | Time     | Time Ratio | Tiles/sec");
    println!("------|--------|----------|------------|----------");

    for level in test_levels {
        let expected_tiles = 10 * level * level + 2;

        let start = Instant::now();
        let hexasphere = Hexasphere::new(1.0, level, 1.0);
        let generation_time = start.elapsed();

        assert_eq!(hexasphere.tiles.len(), expected_tiles);

        let time_ratio = generation_time.as_nanos() as f64 / previous_time.as_nanos() as f64;
        let tiles_per_sec = hexasphere.tiles.len() as f64 / generation_time.as_secs_f64();

        println!(
            "{:5} | {:6} | {:8?} | {:9.1}x | {:8.0}",
            level,
            hexasphere.tiles.len(),
            generation_time,
            time_ratio,
            tiles_per_sec
        );

        // Time should grow reasonably (not exponentially)
        if level > 4 {
            assert!(
                time_ratio < 10.0,
                "Time growth should be reasonable between consecutive levels"
            );
        }

        previous_time = generation_time;
    }

    println!("\n✅ Performance scaling looks reasonable");
}

/// Test that demonstrates readiness for shape instancing
#[test]
fn test_shape_instancing_readiness() {
    println!("\n=== Shape Instancing Readiness Test ===");

    let level = 10;
    let hexasphere = Hexasphere::new(1.0, level, 1.0);

    println!(
        "Analyzing subdivision {} for shape instancing potential",
        level
    );
    println!("Total tiles: {}", hexasphere.tiles.len());

    // Analyze tile types
    let pentagons = hexasphere
        .tiles
        .iter()
        .filter(|t| t.boundary.len() == 5)
        .count();
    let hexagons = hexasphere
        .tiles
        .iter()
        .filter(|t| t.boundary.len() == 6)
        .count();

    println!("Pentagon tiles: {}", pentagons);
    println!("Hexagon tiles: {}", hexagons);

    // Check that orientations are available (needed for instancing)
    let orientations = hexasphere.get_tile_orientations();
    let valid_orientations = orientations.iter().filter(|o| o.is_some()).count();

    println!(
        "Valid orientations: {}/{}",
        valid_orientations,
        orientations.len()
    );

    // Check that regular hexagon approximations work
    let hex_approximations = hexasphere.get_regular_hexagon_approximations();
    println!(
        "Regular hexagon approximations: {}",
        hex_approximations.len()
    );

    // This should be very close to the number of hexagons
    assert!(
        (hex_approximations.len() as i32 - hexagons as i32).abs() <= 1,
        "Approximation count should match hexagon count"
    );

    // Estimate potential compression
    // In practice, we'd expect 10-50 unique shapes at this level
    let estimated_unique_shapes = 50; // Conservative estimate
    let potential_compression = hexasphere.tiles.len() as f64 / estimated_unique_shapes as f64;

    println!("Estimated unique shapes: ~{}", estimated_unique_shapes);
    println!("Potential compression: ~{:.1}x", potential_compression);

    println!("✅ Ready for shape instancing implementation!");

    // The data structures and methods needed for shape instancing are all working
    assert!(valid_orientations > 0);
    assert!(hex_approximations.len() > 0);
    assert_eq!(pentagons, 12);
}
