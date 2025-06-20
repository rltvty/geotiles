use geotiles::Hexasphere;

/// Test that the tile count formula 10n² + 2 is correct for reasonable subdivision levels
#[test]
fn test_tile_count_formula_basic() {
    let test_cases = [
        (0, 12),    // Special case
        (1, 12),    // 10(1) + 2 = 12
        (2, 42),    // 10(4) + 2 = 42
        (3, 92),    // 10(9) + 2 = 92
        (4, 162),   // 10(16) + 2 = 162
        (5, 252),   // 10(25) + 2 = 252
        (10, 1002), // 10(100) + 2 = 1002
    ];

    for (n, expected) in test_cases {
        let hexasphere = Hexasphere::new(1.0, n, 1.0);
        let actual_count = hexasphere.tiles.len();

        assert_eq!(
            actual_count, expected,
            "Tile count formula should be 10n² + 2 for subdivision level {}",
            n
        );
    }
}

/// Test that hexagon count follows the expected pattern
#[test]
fn test_hexagon_count_pattern() {
    for level in [1, 2, 3, 4, 5, 8] {
        let hexasphere = Hexasphere::new(1.0, level, 1.0);
        let hexagon_count = hexasphere
            .tiles
            .iter()
            .filter(|tile| tile.boundary.len() == 6)
            .count();

        let total_tiles = hexasphere.tiles.len();
        let expected_hexagons = total_tiles - 12; // Total minus 12 pentagons

        assert_eq!(
            hexagon_count, expected_hexagons,
            "Hexagon count should be total tiles minus 12 at level {}",
            level
        );
    }
}

/// Test that tiles are either pentagons or hexagons
#[test]
fn test_tile_types_only() {
    for level in [2, 3, 4, 5] {
        let hexasphere = Hexasphere::new(1.0, level, 1.0);

        for (i, tile) in hexasphere.tiles.iter().enumerate() {
            let sides = tile.boundary.len();
            assert!(
                sides == 5 || sides == 6,
                "Tile {} at level {} should have 5 or 6 sides, found {}",
                i,
                level,
                sides
            );
        }
    }
}

/// Test that the corrected documentation examples are accurate
#[test]
fn test_documentation_examples() {
    // Test the updated documentation examples
    let examples = [(0, 12), (1, 12), (2, 42), (3, 92), (4, 162), (5, 252)];

    for (level, expected_tiles) in examples {
        let hexasphere = Hexasphere::new(1.0, level, 1.0);
        assert_eq!(
            hexasphere.tiles.len(),
            expected_tiles,
            "Documentation example for level {} should show {} tiles",
            level,
            expected_tiles
        );
    }
}
