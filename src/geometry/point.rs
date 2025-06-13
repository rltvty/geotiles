//! 3D point representation and operations.

use crate::utils::LatLon;

/// A point in 3D space with coordinates (x, y, z).
///
/// Points are the fundamental building blocks of the geodesic polyhedron.
/// They represent vertices of the underlying triangular mesh and centers of the resulting tiles.
/// Coordinates are rounded to 3 decimal places to match the precision of the original
/// JavaScript implementation and provide consistent hashing behavior.
///
/// # Examples
///
/// ```rust
/// use geotiles::geometry::Point;
///
/// let p1 = Point::new(1.0, 2.0, 3.0);
/// let p2 = Point::new(4.0, 5.0, 6.0);
///
/// // Calculate distance between points
/// let distance = p1.distance_to(&p2);
///
/// // Create intermediate point (50% between p1 and p2)
/// let midpoint = p1.segment(&p2, 0.5);
///
/// // Project point onto sphere of radius 10
/// let mut sphere_point = p1.clone();
/// sphere_point.project(10.0, 1.0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    /// X-coordinate in 3D space
    pub x: f64,
    /// Y-coordinate in 3D space  
    pub y: f64,
    /// Z-coordinate in 3D space
    pub z: f64,
}

impl Point {
    /// Creates a new point with the specified coordinates.
    ///
    /// Coordinates are automatically rounded to 3 decimal places for consistency
    /// with the original JavaScript implementation and to enable reliable equality
    /// comparisons and hashing.
    ///
    /// # Arguments
    ///
    /// * `x` - X-coordinate
    /// * `y` - Y-coordinate  
    /// * `z` - Z-coordinate
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geotiles::geometry::Point;
    ///
    /// let point = Point::new(1.23456, 2.34567, 3.45678);
    /// assert_eq!(point.x, 1.235); // Rounded to 3 decimal places
    /// ```
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            x: (x * 1000.0).round() / 1000.0, // Match JS precision
            y: (y * 1000.0).round() / 1000.0,
            z: (z * 1000.0).round() / 1000.0,
        }
    }

    /// Calculates the Euclidean distance between this point and another point.
    ///
    /// Uses the standard 3D distance formula: √((x₂-x₁)² + (y₂-y₁)² + (z₂-z₁)²)
    ///
    /// # Arguments
    ///
    /// * `other` - The point to calculate distance to
    ///
    /// # Returns
    ///
    /// The distance as a floating-point number
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geotiles::geometry::Point;
    ///
    /// let p1 = Point::new(0.0, 0.0, 0.0);
    /// let p2 = Point::new(3.0, 4.0, 0.0);
    /// assert_eq!(p1.distance_to(&p2), 5.0); // 3-4-5 triangle
    /// ```
    pub fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2))
            .sqrt()
    }

    /// Creates a series of points by subdividing the line segment between this point and another.
    ///
    /// This method is used during the icosahedron subdivision process to create intermediate
    /// vertices along the edges of triangular faces.
    ///
    /// # Arguments
    ///
    /// * `other` - The endpoint of the line segment
    /// * `count` - Number of subdivisions (intermediate points + 1)
    ///
    /// # Returns
    ///
    /// A vector containing `count + 1` points, starting with `self`, ending with `other`,
    /// and containing `count - 1` evenly spaced intermediate points.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geotiles::geometry::Point;
    ///
    /// let p1 = Point::new(0.0, 0.0, 0.0);
    /// let p2 = Point::new(3.0, 0.0, 0.0);
    /// let subdivided = p1.subdivide(&p2, 3);
    ///
    /// assert_eq!(subdivided.len(), 4); // p1, intermediate1, intermediate2, p2
    /// assert_eq!(subdivided[1].x, 1.0); // First intermediate point
    /// assert_eq!(subdivided[2].x, 2.0); // Second intermediate point
    /// ```
    pub fn subdivide(&self, other: &Point, count: usize) -> Vec<Point> {
        if count == 0 {
            return vec![self.clone()];
        }
        
        let mut segments = Vec::with_capacity(count + 1);
        segments.push(self.clone());

        for i in 1..count {
            let t = i as f64 / count as f64;
            let new_point = Point::new(
                self.x * (1.0 - t) + other.x * t,
                self.y * (1.0 - t) + other.y * t,
                self.z * (1.0 - t) + other.z * t,
            );
            segments.push(new_point);
        }

        segments.push(other.clone());
        segments
    }

    /// Creates a point along the line segment between this point and another at a specified percentage.
    ///
    /// This is used to create tile boundaries by positioning boundary points at a certain
    /// percentage of the distance from the tile center to the face centroids.
    ///
    /// # Arguments
    ///
    /// * `other` - The endpoint of the line segment
    /// * `percent` - Position along the segment (0.0 = this point, 1.0 = other point)
    ///   Automatically clamped to range [0.0, 1.0]
    ///
    /// # Returns
    ///
    /// A new point at the specified position along the line segment
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::geometry::Point;
    /// let center = Point::new(0.0, 0.0, 0.0);
    /// let edge = Point::new(10.0, 0.0, 0.0);
    ///
    /// let boundary = center.segment(&edge, 0.8); // 80% from center toward edge
    /// assert_eq!(boundary.x, 8.0); // 80% of the way from center to edge
    /// ```
    pub fn segment(&self, other: &Point, percent: f64) -> Point {
        let percent = percent.clamp(0.0, 1.0);
        Point::new(
            self.x * (1.0 - percent) + other.x * percent,
            self.y * (1.0 - percent) + other.y * percent,
            self.z * (1.0 - percent) + other.z * percent,
        )
    }

    /// Projects this point onto a sphere of the specified radius.
    ///
    /// This operation normalizes the point's direction vector and scales it to the desired radius.
    /// It's the key operation that transforms the subdivided icosahedron into a geodesic sphere.
    /// The `percent` parameter allows for partial projection (useful for animations or debugging).
    ///
    /// # Arguments
    ///
    /// * `radius` - Target radius of the sphere
    /// * `percent` - How much of the projection to apply (0.0 = no change, 1.0 = full projection)
    ///   Automatically clamped to range [0.0, 1.0]
    ///
    /// # Returns
    ///
    /// A mutable reference to self (for method chaining)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geotiles::geometry::Point;
    ///
    /// let mut point = Point::new(3.0, 4.0, 0.0);
    /// point.project(10.0, 1.0); // Project onto sphere of radius 10
    ///
    /// let distance = (point.x.powi(2) + point.y.powi(2) + point.z.powi(2)).sqrt();
    /// assert!((distance - 10.0).abs() < 0.001); // Should be very close to 10.0
    /// ```
    pub fn project(&mut self, radius: f64, percent: f64) -> &mut Self {
        let percent = percent.clamp(0.0, 1.0);
        let mag = (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt();
        let ratio = radius / mag;

        self.x = self.x * ratio * percent;
        self.y = self.y * ratio * percent;
        self.z = self.z * ratio * percent;
        self
    }

    /// Converts this 3D point to latitude and longitude coordinates.
    ///
    /// Assumes the point lies on a sphere centered at the origin. Uses spherical coordinate
    /// conversion with a specific rotation to match the original JavaScript implementation's
    /// coordinate system.
    ///
    /// # Arguments
    ///
    /// * `radius` - The radius of the sphere this point lies on
    ///
    /// # Returns
    ///
    /// A `LatLon` struct containing latitude and longitude in degrees
    ///
    /// # Mathematical Notes
    ///
    /// - Latitude (φ): Calculated as `arccos(y/radius)`, then converted to degrees and adjusted to [-90, 90] range
    /// - Longitude (θ): Calculated as `atan2(x, z)` with additional rotation and wrapping to [-180, 180] range
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Point;
    /// let point = Point::new(10.0, 0.0, 0.0); // Point on equator
    /// let lat_lon = point.to_lat_lon(10.0);
    /// assert!((lat_lon.lat - 0.0).abs() < 0.1); // Near equator
    /// ```
    pub fn to_lat_lon(&self, radius: f64) -> LatLon {
        // For Y-up coordinate system: latitude is angle from XZ plane toward +Y
        let lat_radians = (self.y / radius).asin(); // Direct latitude calculation
        let lon_radians = self.x.atan2(self.z); // Longitude in XZ plane

        LatLon {
            lat: lat_radians.to_degrees(),
            lon: lon_radians.to_degrees(),
        }
    }
}

impl std::fmt::Display for Point {
    /// Formats the point as a comma-separated string of coordinates.
    ///
    /// This format is used as a unique identifier for points in hash maps and
    /// for debugging output.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{}", self.x, self.y, self.z)
    }
}

// Implement Hash and Eq for Point to use as HashMap key
impl std::hash::Hash for Point {
    /// Hashes the point based on its string representation.
    ///
    /// This ensures that points with the same coordinates (after rounding)
    /// will hash to the same value, enabling reliable deduplication in hash maps.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

impl Eq for Point {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let point = Point::new(1.0, 2.0, 3.0);
        assert_eq!(point.x, 1.0);
        assert_eq!(point.y, 2.0);
        assert_eq!(point.z, 3.0);
    }

    #[test]
    fn test_point_precision_rounding() {
        let point = Point::new(1.23456789, 2.34567891, 3.45678912);
        assert_eq!(point.x, 1.235);
        assert_eq!(point.y, 2.346);
        assert_eq!(point.z, 3.457);
    }

    #[test]
    fn test_distance_calculation() {
        let p1 = Point::new(0.0, 0.0, 0.0);
        let p2 = Point::new(3.0, 4.0, 0.0);
        
        let distance = p1.distance_to(&p2);
        assert!((distance - 5.0).abs() < 0.001); // 3-4-5 triangle
    }

    #[test]
    fn test_distance_symmetry() {
        let p1 = Point::new(1.0, 2.0, 3.0);
        let p2 = Point::new(4.0, 6.0, 8.0);
        
        let dist1 = p1.distance_to(&p2);
        let dist2 = p2.distance_to(&p1);
        
        assert!((dist1 - dist2).abs() < 0.001);
    }

    #[test]
    fn test_distance_zero() {
        let p1 = Point::new(1.0, 2.0, 3.0);
        let p2 = Point::new(1.0, 2.0, 3.0);
        
        let distance = p1.distance_to(&p2);
        assert!(distance.abs() < 0.001);
    }

    #[test]
    fn test_subdivide_line() {
        let p1 = Point::new(0.0, 0.0, 0.0);
        let p2 = Point::new(3.0, 0.0, 0.0);
        
        let subdivided = p1.subdivide(&p2, 3);
        assert_eq!(subdivided.len(), 4);
        
        // Check endpoints
        assert_eq!(subdivided[0], p1);
        assert_eq!(subdivided[3], p2);
        
        // Check intermediate points
        assert!((subdivided[1].x - 1.0).abs() < 0.001);
        assert!((subdivided[2].x - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_subdivide_edge_cases() {
        let p1 = Point::new(0.0, 0.0, 0.0);
        let p2 = Point::new(1.0, 1.0, 1.0);
        
        // Single subdivision (just endpoints)
        let single = p1.subdivide(&p2, 1);
        assert_eq!(single.len(), 2);
        assert_eq!(single[0], p1);
        assert_eq!(single[1], p2);
        
        // Zero subdivisions
        let zero = p1.subdivide(&p2, 0);
        assert_eq!(zero.len(), 1);
        assert_eq!(zero[0], p1);
    }

    #[test]
    fn test_segment_midpoint() {
        let p1 = Point::new(0.0, 0.0, 0.0);
        let p2 = Point::new(2.0, 2.0, 2.0);
        
        let midpoint = p1.segment(&p2, 0.5);
        assert!((midpoint.x - 1.0).abs() < 0.001);
        assert!((midpoint.y - 1.0).abs() < 0.001);
        assert!((midpoint.z - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_segment_endpoints() {
        let p1 = Point::new(1.0, 2.0, 3.0);
        let p2 = Point::new(4.0, 5.0, 6.0);
        
        let start = p1.segment(&p2, 0.0); // Should be p1 
        assert!((start.x - p1.x).abs() < 0.001);
        assert!((start.y - p1.y).abs() < 0.001);
        assert!((start.z - p1.z).abs() < 0.001);
        
        let end = p1.segment(&p2, 1.0); // Should be p2
        assert!((end.x - p2.x).abs() < 0.001);
        assert!((end.y - p2.y).abs() < 0.001);
        assert!((end.z - p2.z).abs() < 0.001);
    }

    #[test]
    fn test_segment_quarter_points() {
        let p1 = Point::new(0.0, 0.0, 0.0);
        let p2 = Point::new(4.0, 0.0, 0.0);
        
        let quarter = p1.segment(&p2, 0.25); // 25% toward p2 from p1
        assert!((quarter.x - 1.0).abs() < 0.001);
        assert!(quarter.y.abs() < 0.001);
        assert!(quarter.z.abs() < 0.001);
    }

    #[test]
    fn test_projection_to_unit_sphere() {
        let mut point = Point::new(2.0, 0.0, 0.0);
        point.project(1.0, 1.0);
        
        // Should be normalized to unit sphere
        assert!((point.x - 1.0).abs() < 0.001);
        assert!(point.y.abs() < 0.001);
        assert!(point.z.abs() < 0.001);
        
        // Check magnitude
        let mag = (point.x * point.x + point.y * point.y + point.z * point.z).sqrt();
        assert!((mag - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_projection_to_different_radius() {
        let mut point = Point::new(1.0, 1.0, 1.0);
        let _original_mag = (point.x * point.x + point.y * point.y + point.z * point.z).sqrt();
        
        point.project(5.0, 1.0);
        
        // Should be scaled to radius 5
        let new_mag = (point.x * point.x + point.y * point.y + point.z * point.z).sqrt();
        assert!((new_mag - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_projection_with_partial_percent() {
        let mut point = Point::new(2.0, 0.0, 0.0);
        point.project(1.0, 0.5);
        
        // Should be half the radius
        let mag = (point.x * point.x + point.y * point.y + point.z * point.z).sqrt();
        assert!((mag - 0.5).abs() < 0.001);
        assert!((point.x - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_projection_chaining() {
        let mut point = Point::new(3.0, 4.0, 0.0);
        point.project(1.0, 1.0).project(10.0, 1.0);
        
        // Should chain projections
        let mag = (point.x * point.x + point.y * point.y + point.z * point.z).sqrt();
        assert!((mag - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_lat_lon_conversion_poles() {
        // Test north pole
        let north_pole = Point::new(0.0, 1.0, 0.0);
        let lat_lon = north_pole.to_lat_lon(1.0);
        assert!((lat_lon.lat - 90.0).abs() < 0.1);
        
        // Test south pole  
        let south_pole = Point::new(0.0, -1.0, 0.0);
        let lat_lon = south_pole.to_lat_lon(1.0);
        assert!((lat_lon.lat + 90.0).abs() < 0.1);
    }

    #[test]
    fn test_lat_lon_conversion_equator() {
        // Test points on equator
        let equator_point = Point::new(0.0, 0.0, 1.0);
        let lat_lon = equator_point.to_lat_lon(1.0);
        assert!(lat_lon.lat.abs() < 0.1);
    }

    #[test]
    fn test_lat_lon_range() {
        // Test various points to ensure lat/lon are in valid ranges
        let test_points = vec![
            Point::new(1.0, 0.0, 0.0),
            Point::new(0.0, 1.0, 0.0),
            Point::new(-1.0, 0.0, 0.0),
            Point::new(0.0, -1.0, 0.0),
            Point::new(0.577, 0.577, 0.577), // Equal components
        ];
        
        for point in test_points {
            let lat_lon = point.to_lat_lon(1.0);
            assert!(lat_lon.lat >= -90.0 && lat_lon.lat <= 90.0, 
                "Latitude out of range: {}", lat_lon.lat);
            assert!(lat_lon.lon >= -180.0 && lat_lon.lon <= 180.0,
                "Longitude out of range: {}", lat_lon.lon);
        }
    }

    #[test]
    fn test_point_display() {
        let point = Point::new(1.234, -5.678, 9.012);
        let display_string = point.to_string();
        
        // Should contain the coordinates in comma-separated format
        assert!(display_string.contains("1.234"));
        assert!(display_string.contains("-5.678"));
        assert!(display_string.contains("9.012"));
        assert!(display_string.contains(","));
    }

    #[test]
    fn test_point_equality() {
        let p1 = Point::new(1.0, 2.0, 3.0);
        let p2 = Point::new(1.0, 2.0, 3.0);
        let p3 = Point::new(1.1, 2.0, 3.0);
        
        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }

    #[test]
    fn test_point_hash_consistency() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let p1 = Point::new(1.0, 2.0, 3.0);
        let p2 = Point::new(1.0, 2.0, 3.0);
        
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        
        p1.hash(&mut hasher1);
        p2.hash(&mut hasher2);
        
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_point_clone() {
        let original = Point::new(1.0, 2.0, 3.0);
        let cloned = original.clone();
        
        assert_eq!(original, cloned);
        assert_eq!(original.x, cloned.x);
        assert_eq!(original.y, cloned.y);
        assert_eq!(original.z, cloned.z);
    }

    #[test]
    fn test_point_debug() {
        let point = Point::new(1.0, 2.0, 3.0);
        let debug_string = format!("{:?}", point);
        
        // Debug output should contain the coordinates
        assert!(debug_string.contains("1"));
        assert!(debug_string.contains("2"));
        assert!(debug_string.contains("3"));
    }
}
