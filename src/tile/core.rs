//! Core tile implementation.

use super::orientation::TileOrientation;
use crate::approximation::RegularHexagonParams;
use crate::geometry::Vector3;
use crate::geometry::{Face, Point};
use crate::utils::{calculate_surface_normal, pointing_away_from_origin, triangle_area, LatLon};
use std::collections::HashMap;

/// A polygonal tile on the geodesic sphere surface.
///
/// Tiles are the main elements of the Goldberg polyhedron - they represent the polygonal
/// regions that approximate the sphere surface. Most tiles are hexagons (6 sides), but
/// exactly 12 tiles are pentagons (5 sides) due to the underlying icosahedral structure.
///
/// # Structure
///
/// Each tile consists of:
/// - **Center point**: The vertex from the original geodesic polyhedron
/// - **Boundary**: Ordered vertices forming the polygon perimeter  
/// - **Neighbors**: Adjacent tiles that share edges
///
/// # Tile Types
///
/// - **Hexagons**: 6-sided tiles, make up ~90% of the surface
/// - **Pentagons**: 5-sided tiles, exactly 12 per sphere, located at icosahedral vertices
///
/// # Applications
///
/// Tiles can be used for:
/// - Game board spaces (like on a spherical board game)
/// - Geographic regions (like for planet surface division)
/// - Physics simulation cells
/// - Rendering meshes (each tile becomes a polygon)
/// - Navigation waypoints
///
/// # Examples
///
/// ```rust
/// # use geotiles::{Hexasphere, LatLon};
/// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
/// # let tile = &hexasphere.tiles[0];
/// # let sphere_radius = 10.0;
/// // Analyze a tile
/// if tile.is_hexagon() {
///     let lat_lon = tile.get_lat_lon(sphere_radius);
///     println!("Hexagon at {:.2}°N, {:.2}°E", lat_lon.lat, lat_lon.lon);
/// }
///
/// // Get regular hexagon approximation
/// if let Some(hex_params) = tile.get_regular_hexagon_params() {
///     let vertices = hex_params.generate_vertices();
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Tile {
    /// The center point of this tile (vertex from the geodesic polyhedron)
    pub center_point: Point,
    /// Ordered vertices forming the polygon boundary
    pub boundary: Vec<Point>,
    /// String identifiers of neighboring tiles (resolved to indices after construction)
    pub neighbor_ids: Vec<String>,
    /// Indices of neighboring tiles in the main tiles array
    pub neighbors: Vec<usize>,
}

impl Tile {
    /// Creates a new tile from a center point and surrounding faces.
    ///
    /// This is the core constructor that converts a vertex from the geodesic polyhedron
    /// into a tile of the dual Goldberg polyhedron. The process involves:
    ///
    /// 1. Finding all triangular faces that touch the center point
    /// 2. Using face centroids as tile boundary points
    /// 3. Scaling boundary points toward the center based on `hex_size`
    /// 4. Identifying neighboring tiles from the faces
    /// 5. Fixing the boundary orientation for consistent winding
    ///
    /// # Arguments
    ///
    /// * `center_point` - The vertex that becomes the tile center
    /// * `faces` - Mutable slice of faces that surround this vertex
    /// * `hex_size` - Scale factor for tile size (0.01 to 1.0)
    ///   - 1.0: Tiles touch at their boundaries  
    ///   - 0.5: Tiles are half-size with gaps between them
    ///   - 0.01: Very small tiles with large gaps
    ///
    /// # Mathematical Details
    ///
    /// For each face touching the center point:
    /// - Calculate face centroid C
    /// - Create boundary point B = C + (center - C) × (1 - hex_size)
    /// - This scales the centroid toward the center by the hex_size factor
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::{Face, Point, Tile};
    /// # let center = Point::new(0.1, 0.2, 0.3);
    /// # let mut faces = vec![
    /// #     Face::new(0, center.clone(), Point::new(0.2, 0.3, 0.4), Point::new(0.3, 0.4, 0.5)),
    /// #     Face::new(1, center.clone(), Point::new(0.3, 0.4, 0.5), Point::new(0.4, 0.5, 0.6)),
    /// #     Face::new(2, center.clone(), Point::new(0.4, 0.5, 0.6), Point::new(0.5, 0.6, 0.7)),
    /// #     Face::new(3, center.clone(), Point::new(0.5, 0.6, 0.7), Point::new(0.6, 0.7, 0.8)),
    /// #     Face::new(4, center.clone(), Point::new(0.6, 0.7, 0.8), Point::new(0.7, 0.8, 0.9)),
    /// #     Face::new(5, center.clone(), Point::new(0.7, 0.8, 0.9), Point::new(0.2, 0.3, 0.4)),
    /// # ];
    /// let tile = Tile::new(center, &mut faces, 0.9);
    /// // Creates a tile that's 90% of full size
    /// ```
    pub fn new(center_point: Point, faces: &mut [Face], hex_size: f64) -> Self {
        let hex_size = hex_size.clamp(0.01, 1.0);

        let mut boundary = Vec::new();
        let mut neighbor_hash = HashMap::new();

        // Build boundary and collect neighbors
        for face in faces.iter_mut() {
            // Add boundary point
            let centroid = face.get_centroid().clone();
            boundary.push(center_point.segment(&centroid, hex_size));

            // Collect neighbors
            let other_points = face.get_other_points(&center_point);
            for other_point in other_points {
                neighbor_hash.insert(other_point.to_string(), true);
            }
        }

        let neighbor_ids: Vec<String> = neighbor_hash.into_keys().collect();

        // Fix boundary orientation
        let mut tile = Self {
            center_point: center_point.clone(),
            boundary,
            neighbor_ids,
            neighbors: Vec::new(),
        };

        tile.fix_boundary_orientation();
        tile
    }

    /// Ensures the tile boundary has consistent counter-clockwise winding.
    ///
    /// This method checks if the boundary vertices are oriented correctly by
    /// calculating the surface normal and verifying it points outward from the sphere.
    /// If the normal points inward, the boundary order is reversed.
    ///
    /// # Why This Matters
    ///
    /// Consistent winding order is crucial for:
    /// - Proper lighting calculations (surface normals)
    /// - Correct rendering (front-face culling)
    /// - Physics collision detection
    /// - Area calculations with correct sign
    ///
    /// # Algorithm
    ///
    /// 1. Calculate surface normal using first three boundary points
    /// 2. Check if normal points away from sphere center (outward)
    /// 3. If normal points inward, reverse the boundary vertex order
    fn fix_boundary_orientation(&mut self) {
        if self.boundary.len() >= 3 {
            let normal =
                calculate_surface_normal(&self.boundary[1], &self.boundary[2], &self.boundary[0]);

            if !pointing_away_from_origin(&self.center_point, &normal) {
                self.boundary.reverse();
            }
        }
    }

    /// Converts the tile center to latitude and longitude coordinates.
    ///
    /// This method treats the tile center as a point on a sphere and converts
    /// its 3D coordinates to geographic coordinates. Useful for mapping applications,
    /// coordinate system conversions, or integration with geographic data.
    ///
    /// # Arguments
    ///
    /// * `radius` - The radius of the sphere the tile lies on
    ///
    /// # Returns
    ///
    /// A `LatLon` struct with latitude and longitude in degrees
    ///
    /// # Coordinate System
    ///
    /// - **Latitude**: -90° (South Pole) to +90° (North Pole)
    /// - **Longitude**: -180° to +180° (with 0° at a reference meridian)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// # let tile = &hexasphere.tiles[0];
    /// let lat_lon = tile.get_lat_lon(10.0);
    /// println!("Tile at {:.2}°N, {:.2}°E", lat_lon.lat, lat_lon.lon);
    ///
    /// // Check if tile is in northern hemisphere
    /// if lat_lon.lat > 0.0 {
    ///     println!("Northern hemisphere tile");
    /// }
    /// ```
    pub fn get_lat_lon(&self, radius: f64) -> LatLon {
        self.center_point.to_lat_lon(radius)
    }

    /// Converts a specific boundary point to latitude and longitude coordinates.
    ///
    /// Similar to `get_lat_lon()` but operates on a boundary vertex instead of
    /// the tile center. Useful for getting precise geographic coordinates of
    /// tile corners or edges.
    ///
    /// # Arguments
    ///
    /// * `radius` - The radius of the sphere
    /// * `boundary_num` - Index of the boundary point (0 to boundary.len()-1)
    ///
    /// # Returns
    ///
    /// Some(`LatLon`) if the boundary index is valid, None otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// # let tile = &hexasphere.tiles[0];
    /// // Get coordinates of all boundary points
    /// for i in 0..tile.boundary.len() {
    ///     if let Some(lat_lon) = tile.get_boundary_lat_lon(10.0, i) {
    ///         println!("Vertex {}: {:.2}°N, {:.2}°E", i, lat_lon.lat, lat_lon.lon);
    ///     }
    /// }
    /// ```
    pub fn get_boundary_lat_lon(&self, radius: f64, boundary_num: usize) -> Option<LatLon> {
        self.boundary
            .get(boundary_num)
            .map(|point| point.to_lat_lon(radius))
    }

    /// Creates a smaller version of the tile boundary by scaling toward the center.
    ///
    /// This method generates a new boundary that's scaled down from the original,
    /// creating a visual gap between tiles. Useful for rendering tiles with
    /// visible separation or creating beveled edges.
    ///
    /// # Arguments
    ///
    /// * `scale` - How much to scale down (0.0 = point at center, 1.0 = original size)
    ///   Automatically clamped to [0.0, 1.0]
    ///
    /// # Returns
    ///
    /// A vector of points forming the scaled boundary
    ///
    /// # Mathematical Details
    ///
    /// For each boundary point B and tile center C:
    /// New point = C + (B - C) × scale
    ///
    /// # Visual Effects
    ///
    /// - `scale = 1.0`: Original boundary (tiles touch)
    /// - `scale = 0.8`: 20% gap between tiles  
    /// - `scale = 0.5`: 50% smaller tiles with large gaps
    /// - `scale = 0.0`: All points collapse to center
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// # let tile = &hexasphere.tiles[0];
    /// // Create tile with 10% border gap
    /// let smaller_boundary = tile.scaled_boundary(0.9);
    ///
    /// // Create very small tiles for debugging
    /// let tiny_boundary = tile.scaled_boundary(0.2);
    /// ```
    pub fn scaled_boundary(&self, scale: f64) -> Vec<Point> {
        let scale = scale.clamp(0.0, 1.0);
        self.boundary
            .iter()
            .map(|boundary_point| self.center_point.segment(boundary_point, 1.0 - scale))
            .collect()
    }

    /// Returns true if this is a hexagon (6 sides), false if pentagon (5 sides).
    ///
    /// Hexagons make up the vast majority of tiles (~90%) and are located away
    /// from the 12 icosahedral vertices. This distinction is important for:
    /// - Regular hexagon approximations (only works for hexagons)
    /// - Statistical analysis
    /// - Special handling of pentagonal tiles
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// # let tile = &hexasphere.tiles[0];
    /// if tile.is_hexagon() {
    ///     // Apply hexagon-specific processing
    ///     let regular_params = tile.get_regular_hexagon_params();
    /// } else {
    ///     // Handle pentagon specially
    ///     println!("Found pentagon at icosahedral vertex");
    /// }
    /// ```
    pub fn is_hexagon(&self) -> bool {
        self.boundary.len() == 6
    }

    /// Returns true if this is a pentagon (5 sides), false if hexagon (6 sides).
    ///
    /// Pentagons are special tiles that occur at exactly 12 locations corresponding
    /// to the vertices of the original icosahedron. They're always smaller than
    /// hexagons and create the curvature necessary to wrap a flat tiling around a sphere.
    ///
    /// # Mathematical Significance
    ///
    /// The 12 pentagons are required by Euler's formula (V - E + F = 2) and cannot
    /// be avoided when tiling a sphere. Their locations determine the overall
    /// symmetry and distortion pattern of the geodesic polyhedron.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// let pentagon_count = hexasphere.tiles.iter()
    ///     .filter(|tile| tile.is_pentagon())
    ///     .count();
    /// assert_eq!(pentagon_count, 12); // Always exactly 12
    /// ```
    pub fn is_pentagon(&self) -> bool {
        self.boundary.len() == 5
    }

    /// Calculate the average distance from center to boundary points (approximates radius).
    ///
    /// This provides a measure of the tile's "size" by calculating how far the boundary
    /// extends from the center on average. For a regular hexagon, this would be the
    /// circumradius (center to vertex distance). For irregular tiles, it's an approximation.
    ///
    /// # Returns
    ///
    /// Average distance from center to boundary points, or 0.0 if no boundary exists
    ///
    /// # Use Cases
    ///
    /// - Choosing uniform hexagon size for approximations
    /// - Analyzing size variation across the sphere
    /// - Quality metrics for geodesic construction
    /// - Scaling and normalization calculations
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// # let tile = &hexasphere.tiles[0];
    /// let radius = tile.get_average_radius();
    /// println!("Tile size: {:.3} units", radius);
    ///
    /// // Compare sizes
    /// # let average_size = 1.0;
    /// if radius > average_size * 1.1 {
    ///     println!("This tile is larger than average");
    /// }
    /// ```
    pub fn get_average_radius(&self) -> f64 {
        if self.boundary.is_empty() {
            return 0.0;
        }

        let total_distance: f64 = self
            .boundary
            .iter()
            .map(|point| self.center_point.distance_to(point))
            .sum();

        total_distance / self.boundary.len() as f64
    }

    /// Calculate the average edge length of this tile.
    ///
    /// Measures the average distance between consecutive boundary points, giving
    /// an indication of the tile's perimeter size. For regular polygons, all edges
    /// would be equal. For geodesic tiles, there's some variation due to spherical distortion.
    ///
    /// # Returns
    ///
    /// Average length of boundary edges, or 0.0 if fewer than 2 boundary points exist
    ///
    /// # Mathematical Details
    ///
    /// Calculates the distance between each pair of consecutive boundary points:
    /// - Edge 0: distance(boundary[0], boundary[1])
    /// - Edge 1: distance(boundary[1], boundary[2])
    /// - ...
    /// - Last edge: distance(boundary[n-1], boundary[0]) (wrapping around)
    ///
    /// # Use Cases
    ///
    /// - Analyzing edge length uniformity
    /// - Detecting highly distorted tiles
    /// - Choosing appropriate subdivision levels
    /// - Quality assessment of geodesic construction
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// # let tile = &hexasphere.tiles[0];
    /// let edge_length = tile.get_average_edge_length();
    /// let radius = tile.get_average_radius();
    ///
    /// // For regular hexagon: edge_length ≈ radius (approximately)
    /// let regularity = (edge_length / radius - 1.0).abs();
    /// if regularity < 0.1 {
    ///     println!("This tile is quite regular");
    /// }
    /// ```
    pub fn get_average_edge_length(&self) -> f64 {
        if self.boundary.len() < 2 {
            return 0.0;
        }

        let mut total_length = 0.0;
        for i in 0..self.boundary.len() {
            let next_i = (i + 1) % self.boundary.len();
            total_length += self.boundary[i].distance_to(&self.boundary[next_i]);
        }

        total_length / self.boundary.len() as f64
    }

    /// Get the area of this tile (approximate, using triangulation from center).
    ///
    /// Calculates the surface area of the tile by dividing it into triangles
    /// from the center point to each boundary edge. This gives an approximation
    /// of the actual spherical area.
    ///
    /// # Returns
    ///
    /// Approximate surface area of the tile, or 0.0 if fewer than 3 boundary points
    ///
    /// # Algorithm
    ///
    /// 1. Divide tile into triangles: center + each boundary edge
    /// 2. Calculate area of each triangle using cross product
    /// 3. Sum all triangle areas
    ///
    /// # Accuracy Notes
    ///
    /// - This is a planar approximation of the actual spherical area
    /// - Accuracy decreases for larger tiles or higher curvature
    /// - Good enough for most analysis and comparison purposes
    ///
    /// # Use Cases
    ///
    /// - Comparing tile sizes across the sphere
    /// - Detecting area distortions
    /// - Equal-area analysis
    /// - Statistical studies of geodesic properties
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// # let tile = &hexasphere.tiles[0];
    /// let area = tile.get_area();
    /// println!("Tile covers {:.6} square units", area);
    ///
    /// // Calculate area density
    /// # let radius: f64 = 10.0;
    /// let sphere_area = 4.0 * std::f64::consts::PI * radius.powi(2);
    /// let coverage = area / sphere_area;
    /// println!("This tile covers {:.2}% of sphere", coverage * 100.0);
    /// ```
    pub fn get_area(&self) -> f64 {
        if self.boundary.len() < 3 {
            return 0.0;
        }

        let mut total_area = 0.0;
        for i in 0..self.boundary.len() {
            let next_i = (i + 1) % self.boundary.len();
            // Area of triangle formed by center and two consecutive boundary points
            let triangle_area = triangle_area(
                &self.center_point,
                &self.boundary[i],
                &self.boundary[next_i],
            );
            total_area += triangle_area;
        }

        total_area
    }

    /// Calculate the orientation of this tile for placing a regular hexagon.
    ///
    /// Determines the local coordinate system for this tile, which can be used
    /// to orient regular hexagons or other objects. The orientation is defined
    /// by three orthogonal unit vectors forming a right-handed coordinate system.
    ///
    /// # Returns
    ///
    /// Some(`TileOrientation`) containing the coordinate system vectors, or None if
    /// the tile has no boundary points
    ///
    /// # Coordinate System Definition
    ///
    /// - **Right vector**: Points from center toward first boundary vertex
    /// - **Up vector**: Points outward from sphere surface (surface normal)
    /// - **Forward vector**: Perpendicular to both (completes right-handed system)
    ///
    /// # Algorithm
    ///
    /// 1. Calculate right vector: normalize(first_boundary - center)
    /// 2. Calculate up vector: normalize(center) (sphere normal)
    /// 3. Calculate forward vector: cross(right, up)
    /// 4. Recalculate right: cross(up, forward) (ensure orthogonality)
    ///
    /// # Use Cases
    ///
    /// - Positioning regular hexagon meshes
    /// - Aligning objects with tile orientation
    /// - Creating transformation matrices
    /// - Texture mapping coordinate systems
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// # let tile = &hexasphere.tiles[0];
    /// if let Some(orientation) = tile.get_orientation() {
    ///     let transform = orientation.to_transform_matrix(&tile.center_point);
    ///     
    ///     // Use transform matrix in 3D engine
    ///     # // spawn_hexagon_mesh(transform);
    /// }
    /// ```
    pub fn get_orientation(&self) -> Option<TileOrientation> {
        if self.boundary.is_empty() {
            return None;
        }

        // Use the first boundary point to define the "right" direction
        let first_vertex = &self.boundary[0];

        // Calculate the "right" vector (center to first vertex)
        let right = Vector3::new(
            first_vertex.x - self.center_point.x,
            first_vertex.y - self.center_point.y,
            first_vertex.z - self.center_point.z,
        )
        .normalize();

        // Calculate the "up" vector (normal to sphere surface)
        // For a sphere centered at origin, this is just the center point normalized
        let up = Vector3::new(
            self.center_point.x,
            self.center_point.y,
            self.center_point.z,
        )
        .normalize();

        // Calculate the "forward" vector (cross product of right and up)
        let forward = right.cross(&up).normalize();

        // Recalculate right to ensure orthogonality (cross product of up and forward)
        let right = up.cross(&forward).normalize();

        Some(TileOrientation { right, up, forward })
    }

    /// Get the best regular hexagon parameters for this tile.
    ///
    /// Calculates the position, size, and orientation for a regular hexagon that
    /// best approximates this irregular tile. Only works for hexagonal tiles;
    /// returns None for pentagons since they can't be approximated as regular hexagons.
    ///
    /// # Returns
    ///
    /// Some(`RegularHexagonParams`) if this is a hexagon, None if it's a pentagon
    ///
    /// # Parameter Calculation
    ///
    /// - **Center**: Uses the tile's center point
    /// - **Radius**: Uses the average distance from center to boundary points
    /// - **Orientation**: Uses the tile's computed orientation vectors
    ///
    /// # Quality of Approximation
    ///
    /// The quality depends on:
    /// - How close the original tile is to regular
    /// - Distance from icosahedral vertices (pentagons)
    /// - Subdivision level (higher = more regular)
    ///
    /// # Use Cases
    ///
    /// - Generating regular hexagon meshes for rendering
    /// - Creating uniform tile sizes for gameplay
    /// - Physics collision shapes
    /// - Simplified geometric calculations
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// # let tile = &hexasphere.tiles[0];
    /// if let Some(hex_params) = tile.get_regular_hexagon_params() {
    ///     let vertices = hex_params.generate_vertices();
    ///     println!("Regular hexagon has {} vertices", vertices.len()); // Always 6
    ///     
    ///     // Check how well it approximates the original
    ///     # fn calculate_hexagon_area(radius: f64) -> f64 {
    ///     #     // Area of regular hexagon = (3√3/2) * r²
    ///     #     1.5 * (3.0_f64).sqrt() * radius * radius
    ///     # }
    ///     let regular_area = calculate_hexagon_area(hex_params.radius);
    ///     let original_area = tile.get_area();
    ///     let error = (regular_area - original_area).abs() / original_area;
    ///     println!("Approximation error: {:.1}%", error * 100.0);
    /// } else {
    ///     println!("Can't approximate pentagon as regular hexagon");
    /// }
    /// ```
    pub fn get_regular_hexagon_params(&self) -> Option<RegularHexagonParams> {
        if !self.is_hexagon() {
            return None; // Only works for hexagons
        }

        let orientation = self.get_orientation()?;
        let radius = self.get_average_radius();

        Some(RegularHexagonParams {
            center: self.center_point.clone(),
            radius,
            orientation,
        })
    }
}

impl std::fmt::Display for Tile {
    /// Formats the tile using its center point coordinates.
    ///
    /// This provides a unique string identifier for the tile based on its
    /// center point location. Used for debugging, logging, and as hash keys.
    ///
    /// # Output Format
    ///
    /// Returns the center point coordinates as "x,y,z" (same as Point::Display)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// # let tile = &hexasphere.tiles[0];
    /// # use std::collections::HashMap;
    /// let tile_id = tile.to_string();
    /// println!("Processing tile: {}", tile); // Uses this Display implementation
    ///
    /// // Can be used as a unique identifier
    /// let mut tile_map = HashMap::new();
    /// # let tile_data = 42; // example data
    /// tile_map.insert(tile.to_string(), tile_data);
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.center_point)
    }
}

#[cfg(test)]
mod tests {
    use crate::hexasphere::core::Hexasphere;

    #[test]
    fn test_thick_tiles() {
        let hexasphere = Hexasphere::new(10.0, 2, 0.8);
        let thick_tiles = hexasphere.create_thick_tiles(0.5);

        assert_eq!(thick_tiles.len(), hexasphere.tiles.len());

        // Test first thick tile
        if let Some(thick_tile) = thick_tiles.first() {
            assert_eq!(
                thick_tile.outer_boundary.len(),
                thick_tile.inner_boundary.len()
            );
            assert!(thick_tile.thickness > 0.0);

            let vertices = thick_tile.generate_all_vertices();
            assert!(vertices.vertices.len() > 0);
            assert!(vertices.indices.len() > 0);
            assert_eq!(vertices.indices.len() % 3, 0); // Should be triangles
        }
    }

    #[test]
    fn test_inner_sphere_creation() {
        let outer_sphere = Hexasphere::new(10.0, 2, 0.8);
        let inner_sphere = outer_sphere.create_inner_sphere(9.0);

        assert_eq!(inner_sphere.radius, 9.0);
        assert_eq!(inner_sphere.tiles.len(), outer_sphere.tiles.len());

        // Check that tiles are properly scaled
        for (outer_tile, inner_tile) in outer_sphere.tiles.iter().zip(inner_sphere.tiles.iter()) {
            let outer_distance = (outer_tile.center_point.x.powi(2)
                + outer_tile.center_point.y.powi(2)
                + outer_tile.center_point.z.powi(2))
            .sqrt();
            let inner_distance = (inner_tile.center_point.x.powi(2)
                + inner_tile.center_point.y.powi(2)
                + inner_tile.center_point.z.powi(2))
            .sqrt();

            assert!((outer_distance - 10.0).abs() < 0.1);
            assert!((inner_distance - 9.0).abs() < 0.1);
        }
    }

    #[test]
    fn test_scaled_boundary() {
        let hexasphere = Hexasphere::new(1.0, 2, 1.0);
        let tile = &hexasphere.tiles[0];
        
        let half_scale = tile.scaled_boundary(0.5);
        let full_scale = tile.scaled_boundary(1.0);
        let zero_scale = tile.scaled_boundary(0.0);
        
        assert_eq!(half_scale.len(), full_scale.len());
        assert_eq!(zero_scale.len(), tile.boundary.len());
        
        // Zero scale should collapse all points to center
        for point in &zero_scale {
            assert!((point.x - tile.center_point.x).abs() < 0.001);
            assert!((point.y - tile.center_point.y).abs() < 0.001);
            assert!((point.z - tile.center_point.z).abs() < 0.001);
        }
        
        // Half scale should be smaller than full scale
        for (half, full) in half_scale.iter().zip(full_scale.iter()) {
            let half_dist = tile.center_point.distance_to(half);
            let full_dist = tile.center_point.distance_to(full);
            assert!(half_dist <= full_dist);
        }
    }

    #[test]
    fn test_lat_lon_conversion() {
        let hexasphere = Hexasphere::new(1.0, 2, 1.0);
        let tile = &hexasphere.tiles[0];
        
        let lat_lon = tile.get_lat_lon(1.0);
        assert!(lat_lon.lat >= -90.0 && lat_lon.lat <= 90.0);
        assert!(lat_lon.lon >= -180.0 && lat_lon.lon <= 180.0);
        
        // Test boundary lat/lon
        if let Some(boundary_ll) = tile.get_boundary_lat_lon(1.0, 0) {
            assert!(boundary_ll.lat >= -90.0 && boundary_ll.lat <= 90.0);
            assert!(boundary_ll.lon >= -180.0 && boundary_ll.lon <= 180.0);
        }
        
        // Test invalid boundary index
        assert!(tile.get_boundary_lat_lon(1.0, 100).is_none());
    }

    #[test]
    fn test_tile_measurements() {
        let hexasphere = Hexasphere::new(2.0, 2, 1.0);
        let tile = &hexasphere.tiles[0];
        
        let radius = tile.get_average_radius();
        let edge_length = tile.get_average_edge_length();
        let area = tile.get_area();
        
        assert!(radius > 0.0);
        assert!(edge_length > 0.0);
        assert!(area > 0.0);
        
        // For reasonable geodesic tiles, these should be related
        assert!(edge_length > 0.1 * radius, "Edge length should be reasonable vs radius");
        assert!(edge_length < 5.0 * radius, "Edge length shouldn't be too large vs radius");
        assert!(area > 0.1 * radius * radius, "Area should be reasonable vs radius squared");
    }

    #[test]
    fn test_tile_classification() {
        let hexasphere = Hexasphere::new(1.0, 2, 1.0);
        
        let mut hexagon_count = 0;
        let mut pentagon_count = 0;
        
        for tile in &hexasphere.tiles {
            if tile.is_hexagon() {
                hexagon_count += 1;
                assert_eq!(tile.boundary.len(), 6);
                assert!(!tile.is_pentagon());
            } else if tile.is_pentagon() {
                pentagon_count += 1;
                assert_eq!(tile.boundary.len(), 5);
                assert!(!tile.is_hexagon());
            } else {
                panic!("Tile should be either hexagon or pentagon");
            }
        }
        
        // Should have exactly 12 pentagons
        assert_eq!(pentagon_count, 12);
        assert!(hexagon_count > 0);
        assert_eq!(hexagon_count + pentagon_count, hexasphere.tiles.len());
    }

    #[test]
    fn test_tile_orientation_and_params() {
        let hexasphere = Hexasphere::new(1.0, 2, 1.0);
        
        // Find a hexagonal tile
        let hex_tile = hexasphere.tiles.iter()
            .find(|tile| tile.is_hexagon())
            .expect("Should have hexagonal tiles");
        
        // Test orientation
        let orientation = hex_tile.get_orientation()
            .expect("Hexagon should have orientation");
        
        // Vectors should be roughly unit length
        let right_len = (orientation.right.x.powi(2) + 
                        orientation.right.y.powi(2) + 
                        orientation.right.z.powi(2)).sqrt();
        assert!((right_len - 1.0).abs() < 0.1, "Right vector should be unit length");
        
        // Test regular hexagon params
        let params = hex_tile.get_regular_hexagon_params()
            .expect("Hexagon should have regular params");
        
        assert!(params.radius > 0.0);
        assert_eq!(params.center.x, hex_tile.center_point.x);
        assert_eq!(params.center.y, hex_tile.center_point.y);
        assert_eq!(params.center.z, hex_tile.center_point.z);
        
        // Test pentagon cannot get regular hexagon params
        let pentagon_tile = hexasphere.tiles.iter()
            .find(|tile| tile.is_pentagon())
            .expect("Should have pentagon tiles");
        
        assert!(pentagon_tile.get_regular_hexagon_params().is_none());
    }

    #[test]
    fn test_tile_display() {
        let hexasphere = Hexasphere::new(1.0, 1, 1.0);
        let tile = &hexasphere.tiles[0];
        
        let display_string = tile.to_string();
        
        // Should be the same as center point display
        let center_string = tile.center_point.to_string();
        assert_eq!(display_string, center_string);
        
        // Should contain coordinates
        assert!(display_string.contains(&tile.center_point.x.to_string()) ||
                display_string.contains(&format!("{:.3}", tile.center_point.x)));
    }

    #[test]
    fn test_tile_edge_cases() {
        let hexasphere = Hexasphere::new(0.1, 1, 0.01); // Very small with minimal hex size
        
        // Should still work
        assert!(hexasphere.tiles.len() > 0);
        
        for tile in &hexasphere.tiles {
            // Even tiny tiles should have valid measurements
            let radius = tile.get_average_radius();
            let edge_length = tile.get_average_edge_length();
            let area = tile.get_area();
            
            assert!(radius >= 0.0);
            assert!(edge_length >= 0.0);
            assert!(area >= 0.0);
            
            // Boundary should exist and have correct size
            if tile.is_hexagon() {
                assert_eq!(tile.boundary.len(), 6);
            } else {
                assert_eq!(tile.boundary.len(), 5);
            }
        }
    }
}
