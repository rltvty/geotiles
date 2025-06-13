//! Regular hexagon generation and parameters.

use crate::geometry::Point;
use crate::tile::TileOrientation;
use std::f64::consts::PI;

/// Parameters defining a regular hexagon that approximates an irregular tile.
///
/// This struct contains everything needed to generate a perfectly regular hexagon
/// that closely matches the size, position, and orientation of an irregular tile
/// from the geodesic polyhedron. This is useful when you want consistent hexagon
/// shapes for gameplay, rendering, or other applications.
///
/// # Examples
///
/// ```rust
/// # use geotiles::{Hexasphere, RegularHexagonParams};
/// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
/// # let tile = &hexasphere.tiles[0];
/// if let Some(hex_params) = tile.get_regular_hexagon_params() {
///     let vertices = hex_params.generate_vertices();
///     // Use vertices for rendering, collision detection, etc.
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RegularHexagonParams {
    /// Center position of the hexagon
    pub center: Point,
    /// Radius from center to vertices (circumradius)
    pub radius: f64,
    /// Orientation defining how the hexagon is rotated
    pub orientation: TileOrientation,
}

impl RegularHexagonParams {
    /// Generates the 6 vertices of a regular hexagon with this configuration.
    ///
    /// Creates vertices positioned at 60-degree intervals around the center,
    /// with the first vertex aligned to the orientation's right vector.
    /// All vertices lie on a circle of the specified radius.
    ///
    /// # Returns
    ///
    /// A vector of 6 `Point` objects representing the hexagon vertices in order
    ///
    /// # Vertex Order
    ///
    /// Vertices are ordered counter-clockwise when viewed from the orientation's up vector:
    /// - Vertex 0: Aligned with right vector
    /// - Vertex 1: 60° counter-clockwise from vertex 0
    /// - Vertex 2: 120° counter-clockwise from vertex 0
    /// - etc.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::{Hexasphere, RegularHexagonParams};
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// # let tile = hexasphere.tiles.iter().find(|t| t.is_hexagon()).unwrap();
    /// # let hex_params = tile.get_regular_hexagon_params().unwrap();
    /// let vertices = hex_params.generate_vertices();
    /// assert_eq!(vertices.len(), 6);
    ///
    /// // All vertices should be equidistant from center
    /// for vertex in &vertices {
    ///     let distance = hex_params.center.distance_to(vertex);
    ///     assert!((distance - hex_params.radius).abs() < 0.001);
    /// }
    /// ```
    pub fn generate_vertices(&self) -> Vec<Point> {
        let mut vertices = Vec::with_capacity(6);

        for i in 0..6 {
            let angle = (i as f64) * PI / 3.0; // 60 degrees per vertex

            // Calculate position in local hex coordinates
            let local_x = self.radius * angle.cos();
            let local_y = self.radius * angle.sin();

            // Transform to world coordinates using orientation
            let world_x = self.center.x
                + local_x * self.orientation.right.x
                + local_y * self.orientation.forward.x;
            let world_y = self.center.y
                + local_x * self.orientation.right.y
                + local_y * self.orientation.forward.y;
            let world_z = self.center.z
                + local_x * self.orientation.right.z
                + local_y * self.orientation.forward.z;

            vertices.push(Point::new(world_x, world_y, world_z));
        }

        vertices
    }
}
