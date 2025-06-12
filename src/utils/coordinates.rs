//! Coordinate system utilities and conversions.
/// Latitude and longitude coordinates in degrees.
///
/// Used for converting 3D sphere coordinates to geographic coordinates,
/// which can be useful for mapping applications or coordinate system conversions.
///
/// # Examples
///
/// ```rust
/// use geotiles::LatLon;
/// let lat_lon = LatLon { lat: 40.7128, lon: -74.0060 }; // New York City
/// ```
#[derive(Debug, Clone)]
pub struct LatLon {
    /// Latitude in degrees, ranging from -90 (South Pole) to +90 (North Pole)
    pub lat: f64,
    /// Longitude in degrees, ranging from -180 to +180
    pub lon: f64,
}
