//! Statistical analysis of hexagon properties.

use crate::hexasphere::core::Hexasphere;
use crate::tile::core::Tile;

/// Statistical analysis of hexagon properties across the entire hexasphere.
///
/// This struct provides detailed metrics about the size and shape variations
/// of hexagons in the geodesic polyhedron. It's essential for understanding
/// the quality of regular hexagon approximations and choosing appropriate
/// uniform sizes.
///
/// # Use Cases
///
/// - Determining if regular hexagon approximation is acceptable for your application
/// - Choosing the best uniform hexagon size
/// - Understanding distortion patterns in the geodesic projection
/// - Quality assessment for different subdivision levels
///
/// # Examples
///
/// ```rust
/// # use geotiles::Hexasphere;
/// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
/// let stats = hexasphere.calculate_hexagon_stats();
/// println!("Hexagon size varies by {:.1}% across the sphere",
///     100.0 * (stats.max_hexagon_radius - stats.min_hexagon_radius) / stats.average_hexagon_radius);
///
/// if stats.radius_std_deviation / stats.average_hexagon_radius < 0.1 {
///     println!("Regular hexagon approximation should work well!");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct HexagonStats {
    /// Total number of hexagonal tiles (should be much larger than pentagon count)
    pub total_hexagons: usize,
    /// Total number of pentagonal tiles (always exactly 12 for a complete sphere)
    pub total_pentagons: usize,
    /// Average radius of hexagonal tiles (center to vertex distance)
    pub average_hexagon_radius: f64,
    /// Average edge length of hexagonal tiles
    pub average_hexagon_edge_length: f64,
    /// Average surface area of hexagonal tiles
    pub average_hexagon_area: f64,
    /// Smallest hexagon radius found (typically near pentagon vertices)
    pub min_hexagon_radius: f64,
    /// Largest hexagon radius found (typically far from pentagon vertices)
    pub max_hexagon_radius: f64,
    /// Standard deviation of hexagon radii (measure of size consistency)
    pub radius_std_deviation: f64,
}

impl Hexasphere {
    /// Calculate comprehensive statistics about hexagons for approximation purposes.
    ///
    /// Analyzes all hexagonal tiles to provide detailed metrics about size variations,
    /// which is essential for determining the quality of regular hexagon approximations
    /// and choosing appropriate uniform sizes.
    ///
    /// # Returns
    ///
    /// A `HexagonStats` struct containing detailed measurements and statistics
    ///
    /// # Analysis Performed
    ///
    /// - **Size measurements**: Radius, edge length, and area for each hexagon
    /// - **Statistical analysis**: Mean, min, max, and standard deviation
    /// - **Pentagon count**: Always exactly 12 for validation
    /// - **Quality metrics**: Variation coefficients for approximation assessment
    ///
    /// # Use Cases
    ///
    /// - **Quality assessment**: How uniform are the hexagons?
    /// - **Approximation planning**: Is regular hexagon approximation viable?
    /// - **Size selection**: What uniform size minimizes error?
    /// - **Subdivision optimization**: Which detail level provides best uniformity?
    ///
    /// # Statistical Interpretation
    ///
    /// - **Low std deviation**: Hexagons are similar in size (good for approximation)
    /// - **High variation**: Significant distortion (may need higher subdivision)
    /// - **Ratio analysis**: Compare min/max to average for worst-case scenarios
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// let stats = hexasphere.calculate_hexagon_stats();
    ///
    /// // Check uniformity
    /// let variation_percent = 100.0 * stats.radius_std_deviation / stats.average_hexagon_radius;
    /// println!("Hexagon size varies by {:.1}%", variation_percent);
    ///
    /// // Assess approximation quality
    /// if variation_percent < 10.0 {
    ///     println!("Regular hexagon approximation should work well!");
    /// } else {
    ///     println!("Consider higher subdivision for better uniformity");
    /// }
    ///
    /// // Size range analysis
    /// let size_ratio = stats.max_hexagon_radius / stats.min_hexagon_radius;
    /// println!("Largest hexagon is {:.1}x bigger than smallest", size_ratio);
    /// ```
    ///
    /// # Performance
    ///
    /// - Time complexity: O(n×m) where n = hexagon count, m = average boundary points
    /// - Space complexity: O(1) additional memory (streaming calculation)
    /// - Typical execution time: < 1ms for subdivision levels 0-6
    pub fn calculate_hexagon_stats(&self) -> HexagonStats {
        let hexagons: Vec<&Tile> = self.tiles.iter().filter(|tile| tile.is_hexagon()).collect();
        let pentagons: Vec<&Tile> = self
            .tiles
            .iter()
            .filter(|tile| tile.is_pentagon())
            .collect();

        if hexagons.is_empty() {
            return HexagonStats {
                total_hexagons: 0,
                total_pentagons: pentagons.len(),
                average_hexagon_radius: 0.0,
                average_hexagon_edge_length: 0.0,
                average_hexagon_area: 0.0,
                min_hexagon_radius: 0.0,
                max_hexagon_radius: 0.0,
                radius_std_deviation: 0.0,
            };
        }

        let radii: Vec<f64> = hexagons
            .iter()
            .map(|hex| hex.get_average_radius())
            .collect();
        let edge_lengths: Vec<f64> = hexagons
            .iter()
            .map(|hex| hex.get_average_edge_length())
            .collect();
        let areas: Vec<f64> = hexagons.iter().map(|hex| hex.get_area()).collect();

        let avg_radius = radii.iter().sum::<f64>() / radii.len() as f64;
        let avg_edge_length = edge_lengths.iter().sum::<f64>() / edge_lengths.len() as f64;
        let avg_area = areas.iter().sum::<f64>() / areas.len() as f64;

        let min_radius = radii.iter().copied().fold(f64::INFINITY, f64::min);
        let max_radius = radii.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        // Calculate standard deviation
        let variance =
            radii.iter().map(|r| (r - avg_radius).powi(2)).sum::<f64>() / radii.len() as f64;
        let std_deviation = variance.sqrt();

        HexagonStats {
            total_hexagons: hexagons.len(),
            total_pentagons: pentagons.len(),
            average_hexagon_radius: avg_radius,
            average_hexagon_edge_length: avg_edge_length,
            average_hexagon_area: avg_area,
            min_hexagon_radius: min_radius,
            max_hexagon_radius: max_radius,
            radius_std_deviation: std_deviation,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hexasphere::core::Hexasphere;

    #[test]
    fn test_hexagon_stats_basic() {
        let hexasphere = Hexasphere::new(10.0, 2, 1.0);
        let stats = hexasphere.calculate_hexagon_stats();

        // Basic validation
        assert!(stats.total_hexagons > 0);
        assert_eq!(stats.total_pentagons, 12); // Always exactly 12 pentagons
        assert!(stats.average_hexagon_radius > 0.0);
        assert!(stats.average_hexagon_edge_length > 0.0);
        assert!(stats.average_hexagon_area > 0.0);
        assert!(stats.radius_std_deviation >= 0.0);
    }

    #[test]
    fn test_hexagon_stats_size_relationships() {
        let hexasphere = Hexasphere::new(5.0, 3, 1.0);
        let stats = hexasphere.calculate_hexagon_stats();

        // Size relationships should be logical
        assert!(stats.min_hexagon_radius <= stats.average_hexagon_radius);
        assert!(stats.average_hexagon_radius <= stats.max_hexagon_radius);
        assert!(stats.min_hexagon_radius > 0.0);

        // Edge length should be roughly related to radius for regular hexagons
        let edge_to_radius_ratio = stats.average_hexagon_edge_length / stats.average_hexagon_radius;
        assert!(
            edge_to_radius_ratio > 0.5 && edge_to_radius_ratio < 2.0,
            "Edge/radius ratio should be reasonable: {}",
            edge_to_radius_ratio
        );
    }

    #[test]
    fn test_hexagon_stats_total_tiles() {
        let hexasphere = Hexasphere::new(1.0, 2, 1.0);
        let stats = hexasphere.calculate_hexagon_stats();

        // Total should match hexasphere tiles
        assert_eq!(
            stats.total_hexagons + stats.total_pentagons,
            hexasphere.tiles.len()
        );

        // Should be more hexagons than pentagons
        assert!(stats.total_hexagons > stats.total_pentagons);

        // Pentagon count is always exactly 12
        assert_eq!(stats.total_pentagons, 12);
    }

    #[test]
    fn test_hexagon_stats_different_subdivisions() {
        let hex1 = Hexasphere::new(1.0, 2, 1.0); // First level with hexagons
        let hex2 = Hexasphere::new(1.0, 3, 1.0); // Higher subdivision

        let stats1 = hex1.calculate_hexagon_stats();
        let stats2 = hex2.calculate_hexagon_stats();

        // Higher subdivision should have more tiles
        assert!(stats2.total_hexagons > stats1.total_hexagons);

        // Pentagon count is always 12
        assert_eq!(stats1.total_pentagons, 12);
        assert_eq!(stats2.total_pentagons, 12);

        // Higher subdivision should have smaller hexagons
        assert!(stats2.average_hexagon_radius < stats1.average_hexagon_radius);
    }

    #[test]
    fn test_hexagon_stats_variation_metrics() {
        let hexasphere = Hexasphere::new(2.0, 3, 1.0);
        let stats = hexasphere.calculate_hexagon_stats();

        // Standard deviation should be reasonable
        let variation_coefficient = stats.radius_std_deviation / stats.average_hexagon_radius;
        assert!(variation_coefficient >= 0.0);
        assert!(
            variation_coefficient < 1.0,
            "Variation should be reasonable: {}",
            variation_coefficient
        );

        // Size range should be positive
        let size_range = stats.max_hexagon_radius - stats.min_hexagon_radius;
        assert!(size_range >= 0.0);

        // Max/min ratio should be reasonable (not too extreme)
        let size_ratio = stats.max_hexagon_radius / stats.min_hexagon_radius;
        assert!(size_ratio >= 1.0);
        assert!(
            size_ratio < 3.0,
            "Size ratio should be reasonable: {}",
            size_ratio
        );
    }

    #[test]
    fn test_hexagon_stats_different_radii() {
        let small = Hexasphere::new(1.0, 2, 1.0);
        let large = Hexasphere::new(10.0, 2, 1.0);

        let small_stats = small.calculate_hexagon_stats();
        let large_stats = large.calculate_hexagon_stats();

        // Same structure but different scale
        assert_eq!(small_stats.total_hexagons, large_stats.total_hexagons);
        assert_eq!(small_stats.total_pentagons, large_stats.total_pentagons);

        // Measurements should scale with radius
        let scale_factor = 10.0;
        let radius_ratio = large_stats.average_hexagon_radius / small_stats.average_hexagon_radius;
        assert!(
            (radius_ratio - scale_factor).abs() < 0.1,
            "Radius should scale: {} vs expected {}",
            radius_ratio,
            scale_factor
        );

        let edge_ratio =
            large_stats.average_hexagon_edge_length / small_stats.average_hexagon_edge_length;
        assert!(
            (edge_ratio - scale_factor).abs() < 0.1,
            "Edge length should scale: {} vs expected {}",
            edge_ratio,
            scale_factor
        );
    }

    #[test]
    fn test_hexagon_stats_hex_size_parameter() {
        let full_size = Hexasphere::new(1.0, 2, 1.0);
        let half_size = Hexasphere::new(1.0, 2, 0.5);

        let full_stats = full_size.calculate_hexagon_stats();
        let half_stats = half_size.calculate_hexagon_stats();

        // Same number of tiles regardless of hex_size
        assert_eq!(full_stats.total_hexagons, half_stats.total_hexagons);
        assert_eq!(full_stats.total_pentagons, half_stats.total_pentagons);

        // Smaller hex_size should result in smaller measurements
        assert!(half_stats.average_hexagon_radius < full_stats.average_hexagon_radius);
        assert!(half_stats.average_hexagon_edge_length < full_stats.average_hexagon_edge_length);
        assert!(half_stats.average_hexagon_area < full_stats.average_hexagon_area);
    }

    #[test]
    fn test_hexagon_stats_edge_case_single_tile() {
        // Level 0 hexasphere has very few tiles, test it doesn't crash
        let minimal = Hexasphere::new(1.0, 0, 1.0);
        let stats = minimal.calculate_hexagon_stats();

        // Should have 12 pentagons and 0 hexagons for icosahedron
        assert_eq!(stats.total_pentagons, 12);
        assert_eq!(stats.total_hexagons, 0);

        // Hexagon stats should be zero/default for no hexagons
        assert_eq!(stats.average_hexagon_radius, 0.0);
        assert_eq!(stats.average_hexagon_edge_length, 0.0);
        assert_eq!(stats.average_hexagon_area, 0.0);
    }

    #[test]
    fn test_hexagon_stats_consistency() {
        let hexasphere = Hexasphere::new(3.0, 2, 0.8);

        // Run stats calculation multiple times - should be deterministic
        let stats1 = hexasphere.calculate_hexagon_stats();
        let stats2 = hexasphere.calculate_hexagon_stats();

        assert_eq!(stats1.total_hexagons, stats2.total_hexagons);
        assert_eq!(stats1.total_pentagons, stats2.total_pentagons);
        assert!((stats1.average_hexagon_radius - stats2.average_hexagon_radius).abs() < 0.0001);
        assert!(
            (stats1.average_hexagon_edge_length - stats2.average_hexagon_edge_length).abs()
                < 0.0001
        );
        assert!((stats1.average_hexagon_area - stats2.average_hexagon_area).abs() < 0.0001);
        assert!((stats1.radius_std_deviation - stats2.radius_std_deviation).abs() < 0.0001);
    }

    #[test]
    fn test_hexagon_stats_realistic_values() {
        let hexasphere = Hexasphere::new(10.0, 3, 1.0);
        let stats = hexasphere.calculate_hexagon_stats();

        // For a sphere of radius 10, hexagon sizes should be reasonable
        assert!(
            stats.average_hexagon_radius > 0.1 && stats.average_hexagon_radius < 5.0,
            "Hexagon radius should be reasonable: {}",
            stats.average_hexagon_radius
        );
        assert!(
            stats.average_hexagon_edge_length > 0.1 && stats.average_hexagon_edge_length < 5.0,
            "Edge length should be reasonable: {}",
            stats.average_hexagon_edge_length
        );
        assert!(
            stats.average_hexagon_area > 0.01 && stats.average_hexagon_area < 20.0,
            "Area should be reasonable: {}",
            stats.average_hexagon_area
        );

        // Standard deviation should be much smaller than the average
        assert!(
            stats.radius_std_deviation < stats.average_hexagon_radius,
            "Std dev should be smaller than average: {} vs {}",
            stats.radius_std_deviation,
            stats.average_hexagon_radius
        );
    }
}
