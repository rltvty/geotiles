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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Point, Vector3};
    use crate::hexasphere::core::Hexasphere;
    use crate::tile::TileOrientation;

    #[test]
    fn test_generate_vertices_basic() {
        let center = Point::new(0.0, 0.0, 0.0);
        let radius = 1.0;
        let orientation = TileOrientation::default();
        
        let params = RegularHexagonParams {
            center,
            radius,
            orientation,
        };
        
        let vertices = params.generate_vertices();
        assert_eq!(vertices.len(), 6);
        
        // All vertices should be equidistant from center
        for vertex in &vertices {
            let distance = params.center.distance_to(vertex);
            assert!((distance - radius).abs() < 0.001, 
                "Vertex distance {} should equal radius {}", distance, radius);
        }
    }

    #[test]
    fn test_generate_vertices_angles() {
        let center = Point::new(0.0, 0.0, 0.0);
        let radius = 2.0;
        let orientation = TileOrientation::default(); // Identity orientation
        
        let params = RegularHexagonParams {
            center,
            radius,
            orientation,
        };
        
        let vertices = params.generate_vertices();
        
        // Check specific vertex positions for identity orientation
        // Vertex 0 should be at (radius, 0, 0)
        assert!((vertices[0].x - radius).abs() < 0.001);
        assert!(vertices[0].y.abs() < 0.001);
        assert!(vertices[0].z.abs() < 0.001);
        
        // Vertices should be at 60-degree intervals
        for (i, vertex) in vertices.iter().enumerate() {
            let expected_angle = (i as f64) * std::f64::consts::PI / 3.0;
            let expected_x = radius * expected_angle.cos();
            let expected_y = radius * expected_angle.sin();
            
            assert!((vertex.x - expected_x).abs() < 0.001,
                "Vertex {} x: {} vs expected {}", i, vertex.x, expected_x);
            assert!((vertex.y - expected_y).abs() < 0.001,
                "Vertex {} y: {} vs expected {}", i, vertex.y, expected_y);
            assert!(vertex.z.abs() < 0.001,
                "Vertex {} z should be 0: {}", i, vertex.z);
        }
    }

    #[test]
    fn test_generate_vertices_different_radius() {
        let center = Point::new(0.0, 0.0, 0.0);
        let orientation = TileOrientation::default();
        
        let small_params = RegularHexagonParams {
            center: center.clone(),
            radius: 1.0,
            orientation: orientation.clone(),
        };
        
        let large_params = RegularHexagonParams {
            center,
            radius: 3.0,
            orientation,
        };
        
        let small_vertices = small_params.generate_vertices();
        let large_vertices = large_params.generate_vertices();
        
        // Large hexagon vertices should be 3x farther from center
        for (small, large) in small_vertices.iter().zip(large_vertices.iter()) {
            let small_dist = small_params.center.distance_to(small);
            let large_dist = large_params.center.distance_to(large);
            
            assert!((large_dist / small_dist - 3.0).abs() < 0.001,
                "Distance ratio should be 3.0: {}", large_dist / small_dist);
        }
    }

    #[test]
    fn test_generate_vertices_different_center() {
        let orientation = TileOrientation::default();
        let radius = 1.0;
        
        let origin_params = RegularHexagonParams {
            center: Point::new(0.0, 0.0, 0.0),
            radius,
            orientation: orientation.clone(),
        };
        
        let offset_params = RegularHexagonParams {
            center: Point::new(5.0, 3.0, -2.0),
            radius,
            orientation,
        };
        
        let origin_vertices = origin_params.generate_vertices();
        let offset_vertices = offset_params.generate_vertices();
        
        // Offset vertices should be translated by the center difference
        let offset = Point::new(5.0, 3.0, -2.0);
        
        for (origin, offset_vert) in origin_vertices.iter().zip(offset_vertices.iter()) {
            assert!((offset_vert.x - origin.x - offset.x).abs() < 0.001);
            assert!((offset_vert.y - origin.y - offset.y).abs() < 0.001);
            assert!((offset_vert.z - origin.z - offset.z).abs() < 0.001);
        }
    }

    #[test]
    fn test_generate_vertices_custom_orientation() {
        let center = Point::new(0.0, 0.0, 0.0);
        let radius = 1.0;
        
        // 90-degree rotation: right becomes forward, forward becomes -right
        let orientation = TileOrientation {
            right: Vector3::new(0.0, 1.0, 0.0),    // Y axis
            up: Vector3::new(0.0, 0.0, 1.0),       // Z axis
            forward: Vector3::new(-1.0, 0.0, 0.0), // -X axis
        };
        
        let params = RegularHexagonParams {
            center,
            radius,
            orientation,
        };
        
        let vertices = params.generate_vertices();
        
        // First vertex should align with the right vector (Y axis)
        assert!(vertices[0].x.abs() < 0.001, "X should be ~0: {}", vertices[0].x);
        assert!((vertices[0].y - radius).abs() < 0.001, "Y should be radius: {}", vertices[0].y);
        assert!(vertices[0].z.abs() < 0.001, "Z should be ~0: {}", vertices[0].z);
    }

    #[test]
    fn test_hexasphere_regular_hexagon_params() {
        let hexasphere = Hexasphere::new(1.0, 2, 1.0);
        
        // Find a hexagonal tile
        let hex_tile = hexasphere.tiles.iter()
            .find(|tile| tile.is_hexagon())
            .expect("Should have hexagonal tiles");
        
        let params = hex_tile.get_regular_hexagon_params()
            .expect("Hexagon should have regular params");
        
        let vertices = params.generate_vertices();
        assert_eq!(vertices.len(), 6);
        
        // Verify all vertices are the correct distance from center
        for vertex in &vertices {
            let distance = params.center.distance_to(vertex);
            assert!((distance - params.radius).abs() < 0.001,
                "Distance should match radius: {} vs {}", distance, params.radius);
        }
        
        // Verify radius is reasonable for the tile
        let tile_radius = hex_tile.get_average_radius();
        assert!((params.radius - tile_radius).abs() < 0.1,
            "Param radius should match tile radius: {} vs {}", params.radius, tile_radius);
    }

    #[test]
    fn test_regular_hexagon_edge_lengths() {
        let center = Point::new(0.0, 0.0, 0.0);
        let radius = 2.0;
        let orientation = TileOrientation::default();
        
        let params = RegularHexagonParams {
            center,
            radius,
            orientation,
        };
        
        let vertices = params.generate_vertices();
        
        // Check edge lengths are consistent
        let mut edge_lengths = Vec::new();
        for i in 0..6 {
            let next_i = (i + 1) % 6;
            let edge_length = vertices[i].distance_to(&vertices[next_i]);
            edge_lengths.push(edge_length);
        }
        
        // All edges should be the same length
        let first_length = edge_lengths[0];
        for (i, &length) in edge_lengths.iter().enumerate() {
            assert!((length - first_length).abs() < 0.001,
                "Edge {} length {} should match first edge {}", i, length, first_length);
        }
        
        // For regular hexagon: edge_length = radius (approximately)
        assert!((first_length - radius).abs() < 0.001,
            "Edge length {} should equal radius {}", first_length, radius);
    }

    #[test]
    fn test_regular_hexagon_area_calculation() {
        let center = Point::new(0.0, 0.0, 0.0);
        let radius = 1.0;
        let orientation = TileOrientation::default();
        
        let params = RegularHexagonParams {
            center: center.clone(),
            radius,
            orientation,
        };
        
        let vertices = params.generate_vertices();
        
        // Calculate area using triangulation from center
        let mut total_area = 0.0;
        for i in 0..6 {
            let next_i = (i + 1) % 6;
            
            // Area of triangle from center to two consecutive vertices
            let v1 = &vertices[i];
            let v2 = &vertices[next_i];
            
            // Using cross product for triangle area
            let edge1_x = v1.x - center.x;
            let edge1_y = v1.y - center.y;
            let edge2_x = v2.x - center.x;
            let edge2_y = v2.y - center.y;
            
            let triangle_area = 0.5 * (edge1_x * edge2_y - edge1_y * edge2_x).abs();
            total_area += triangle_area;
        }
        
        // Expected area for regular hexagon: (3√3/2) * r²
        let expected_area = 1.5 * (3.0_f64).sqrt() * radius * radius;
        assert!((total_area - expected_area).abs() < 0.001,
            "Calculated area {} should match expected {}", total_area, expected_area);
    }

    #[test]
    fn test_regular_hexagon_symmetry() {
        let center = Point::new(1.0, 2.0, 3.0);
        let radius = 2.5;
        let orientation = TileOrientation::default();
        
        let params = RegularHexagonParams {
            center,
            radius,
            orientation,
        };
        
        let vertices = params.generate_vertices();
        
        // Opposite vertices should be equidistant from center
        for i in 0..3 {
            let vertex_a = &vertices[i];
            let vertex_b = &vertices[i + 3];
            
            let dist_a = params.center.distance_to(vertex_a);
            let dist_b = params.center.distance_to(vertex_b);
            
            assert!((dist_a - dist_b).abs() < 0.001,
                "Opposite vertices should be equidistant: {} vs {}", dist_a, dist_b);
            
            // They should be roughly opposite (2 * radius apart)
            let separation = vertex_a.distance_to(vertex_b);
            assert!((separation - 2.0 * radius).abs() < 0.001,
                "Opposite vertices should be 2*radius apart: {} vs {}", separation, 2.0 * radius);
        }
    }
}
