//! Tile orientation and coordinate system calculations.

use crate::geometry::{Point, Vector3};

/// Orientation information for a tile, defining its local coordinate system.
///
/// This struct contains three orthogonal unit vectors that define how a tile is oriented
/// in 3D space. These vectors can be used to create rotation matrices for positioning
/// regular hexagon meshes or other objects aligned with the tile's orientation.
///
/// # Coordinate System
///
/// - `right`: Points toward the first boundary vertex of the tile
/// - `up`: Points outward from the sphere surface (surface normal)
/// - `forward`: Perpendicular to both right and up (completes right-handed system)
///
/// # Examples
///
/// ```rust
/// # use geotiles::Hexasphere;
/// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
/// # let tile = &hexasphere.tiles[0];
/// // Get orientation for a tile
/// if let Some(orientation) = tile.get_orientation() {
///     let rotation_matrix = orientation.to_rotation_matrix();
///     let transform_matrix = orientation.to_transform_matrix(&tile.center_point);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TileOrientation {
    /// Right vector (toward first boundary vertex)
    pub right: Vector3,
    /// Up vector (outward surface normal)
    pub up: Vector3,
    /// Forward vector (perpendicular to right and up)
    pub forward: Vector3,
}

impl TileOrientation {
    /// Converts the orientation to a 3×3 rotation matrix in row-major order.
    ///
    /// The rotation matrix can be used to transform vectors from local tile coordinates
    /// to world coordinates. Each row represents one of the orientation vectors.
    ///
    /// # Returns
    ///
    /// A 9-element array representing the rotation matrix:
    /// ```text
    /// [right.x,   up.x,   forward.x,
    ///  right.y,   up.y,   forward.y,
    ///  right.z,   up.z,   forward.z]
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// # let tile = &hexasphere.tiles[0];
    /// # let orientation = tile.get_orientation().unwrap();
    /// let matrix = orientation.to_rotation_matrix();
    /// // Use with graphics libraries that expect 3x3 rotation matrices
    /// ```
    pub fn to_rotation_matrix(&self) -> [f64; 9] {
        [
            self.right.x,
            self.up.x,
            self.forward.x,
            self.right.y,
            self.up.y,
            self.forward.y,
            self.right.z,
            self.up.z,
            self.forward.z,
        ]
    }

    /// Converts the orientation to a 4×4 transformation matrix with translation.
    ///
    /// This combines the rotation matrix with a translation vector to create a complete
    /// transformation matrix suitable for 3D graphics pipelines.
    ///
    /// # Arguments
    ///
    /// * `translation` - The position where the transformed object should be placed
    ///
    /// # Returns
    ///
    /// A 16-element array representing the transformation matrix in row-major order:
    /// ```text
    /// [right.x,   up.x,   forward.x,   translation.x,
    ///  right.y,   up.y,   forward.y,   translation.y,
    ///  right.z,   up.z,   forward.z,   translation.z,
    ///  0.0,       0.0,    0.0,         1.0]
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// # let tile = &hexasphere.tiles[0];
    /// # let orientation = tile.get_orientation().unwrap();
    /// let transform = orientation.to_transform_matrix(&tile.center_point);
    /// // Use with 3D engines like Bevy, Three.js, etc.
    /// ```
    pub fn to_transform_matrix(&self, translation: &Point) -> [f64; 16] {
        [
            self.right.x,
            self.up.x,
            self.forward.x,
            translation.x,
            self.right.y,
            self.up.y,
            self.forward.y,
            translation.y,
            self.right.z,
            self.up.z,
            self.forward.z,
            translation.z,
            0.0,
            0.0,
            0.0,
            1.0,
        ]
    }
}

impl Default for TileOrientation {
    /// Creates a default orientation aligned with coordinate axes.
    ///
    /// For hexagon generation in the XY-plane:
    /// - `right` points along +X axis (for hexagon cos component)
    /// - `forward` points along +Y axis (for hexagon sin component)  
    /// - `up` points along +Z axis (normal to hexagon plane)
    fn default() -> Self {
        Self {
            right: Vector3::new(1.0, 0.0, 0.0),   // +X axis
            up: Vector3::new(0.0, 0.0, 1.0),      // +Z axis
            forward: Vector3::new(0.0, 1.0, 0.0), // +Y axis
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Point, Vector3};
    use crate::hexasphere::core::Hexasphere;

    #[test]
    fn test_tile_orientation_basic() {
        let right = Vector3::new(1.0, 0.0, 0.0);
        let up = Vector3::new(0.0, 1.0, 0.0);
        let forward = Vector3::new(0.0, 0.0, 1.0);
        let orientation = TileOrientation { right, up, forward };

        assert_eq!(orientation.right.x, 1.0);
        assert_eq!(orientation.up.y, 1.0);
        assert_eq!(orientation.forward.z, 1.0);
    }

    #[test]
    fn test_default_orientation() {
        let orientation = TileOrientation::default();

        // Should have coordinate axes for hexagon generation
        assert_eq!(orientation.right, Vector3::new(1.0, 0.0, 0.0)); // +X axis
        assert_eq!(orientation.up, Vector3::new(0.0, 0.0, 1.0)); // +Z axis
        assert_eq!(orientation.forward, Vector3::new(0.0, 1.0, 0.0)); // +Y axis
    }

    #[test]
    fn test_to_rotation_matrix() {
        let orientation = TileOrientation::default();
        let matrix = orientation.to_rotation_matrix();

        // Should reflect the coordinate system: right=+X, up=+Z, forward=+Y
        let expected = [
            1.0, 0.0, 0.0, // right vector
            0.0, 0.0, 1.0, // up vector
            0.0, 1.0, 0.0, // forward vector
        ];

        for (i, (&actual, &expected)) in matrix.iter().zip(expected.iter()).enumerate() {
            assert!(
                (actual - expected).abs() < 0.001,
                "Matrix element {} differs: {} vs {}",
                i,
                actual,
                expected
            );
        }
    }

    #[test]
    fn test_to_transform_matrix() {
        let orientation = TileOrientation::default();
        let translation = Point::new(2.0, 3.0, 4.0);
        let matrix = orientation.to_transform_matrix(&translation);

        // Should reflect coordinate system with translation
        let expected = [
            1.0, 0.0, 0.0, 2.0, // right + translation.x
            0.0, 0.0, 1.0, 3.0, // up + translation.y
            0.0, 1.0, 0.0, 4.0, // forward + translation.z
            0.0, 0.0, 0.0, 1.0, // homogeneous row
        ];

        for (i, (&actual, &expected)) in matrix.iter().zip(expected.iter()).enumerate() {
            assert!(
                (actual - expected).abs() < 0.001,
                "Transform matrix element {} differs: {} vs {}",
                i,
                actual,
                expected
            );
        }
    }

    #[test]
    fn test_custom_orientation_matrix() {
        let right = Vector3::new(0.0, 1.0, 0.0); // Y axis
        let up = Vector3::new(0.0, 0.0, 1.0); // Z axis
        let forward = Vector3::new(1.0, 0.0, 0.0); // X axis
        let orientation = TileOrientation { right, up, forward };

        let matrix = orientation.to_rotation_matrix();
        let expected = [
            0.0, 0.0, 1.0, // right vector (Y axis)
            1.0, 0.0, 0.0, // up vector (Z axis)
            0.0, 1.0, 0.0, // forward vector (X axis)
        ];

        for (i, (&actual, &expected)) in matrix.iter().zip(expected.iter()).enumerate() {
            assert!(
                (actual - expected).abs() < 0.001,
                "Custom matrix element {} differs: {} vs {}",
                i,
                actual,
                expected
            );
        }
    }

    #[test]
    fn test_hexasphere_tile_orientations() {
        let hexasphere = Hexasphere::new(1.0, 2, 1.0);

        // Test that all tiles can get orientations
        let mut orientation_count = 0;
        for tile in &hexasphere.tiles {
            if let Some(orientation) = tile.get_orientation() {
                orientation_count += 1;

                // Check that vectors are roughly unit length
                let right_len = (orientation.right.x.powi(2)
                    + orientation.right.y.powi(2)
                    + orientation.right.z.powi(2))
                .sqrt();
                let up_len = (orientation.up.x.powi(2)
                    + orientation.up.y.powi(2)
                    + orientation.up.z.powi(2))
                .sqrt();
                let forward_len = (orientation.forward.x.powi(2)
                    + orientation.forward.y.powi(2)
                    + orientation.forward.z.powi(2))
                .sqrt();

                assert!(
                    (right_len - 1.0).abs() < 0.1,
                    "Right vector should be unit length"
                );
                assert!(
                    (up_len - 1.0).abs() < 0.1,
                    "Up vector should be unit length"
                );
                assert!(
                    (forward_len - 1.0).abs() < 0.1,
                    "Forward vector should be unit length"
                );

                // Test matrix generation doesn't panic
                let _rotation_matrix = orientation.to_rotation_matrix();
                let _transform_matrix = orientation.to_transform_matrix(&tile.center_point);
            }
        }

        // Should get orientations for all tiles
        assert_eq!(orientation_count, hexasphere.tiles.len());
    }

    #[test]
    fn test_orthogonality() {
        let hexasphere = Hexasphere::new(1.0, 1, 1.0);
        let tile = &hexasphere.tiles[0];

        if let Some(orientation) = tile.get_orientation() {
            // Test that vectors are roughly orthogonal (dot product ≈ 0)
            let right_up_dot = orientation.right.x * orientation.up.x
                + orientation.right.y * orientation.up.y
                + orientation.right.z * orientation.up.z;
            let right_forward_dot = orientation.right.x * orientation.forward.x
                + orientation.right.y * orientation.forward.y
                + orientation.right.z * orientation.forward.z;
            let up_forward_dot = orientation.up.x * orientation.forward.x
                + orientation.up.y * orientation.forward.y
                + orientation.up.z * orientation.forward.z;

            assert!(
                right_up_dot.abs() < 0.1,
                "Right and Up should be orthogonal"
            );
            assert!(
                right_forward_dot.abs() < 0.1,
                "Right and Forward should be orthogonal"
            );
            assert!(
                up_forward_dot.abs() < 0.1,
                "Up and Forward should be orthogonal"
            );
        }
    }

    #[test]
    fn test_matrix_dimensions() {
        let orientation = TileOrientation::default();
        let translation = Point::new(0.0, 0.0, 0.0);

        let rotation_matrix = orientation.to_rotation_matrix();
        let transform_matrix = orientation.to_transform_matrix(&translation);

        assert_eq!(rotation_matrix.len(), 9); // 3x3 matrix
        assert_eq!(transform_matrix.len(), 16); // 4x4 matrix
    }

    #[test]
    fn test_different_translations() {
        let orientation = TileOrientation::default();

        let pos1 = Point::new(1.0, 2.0, 3.0);
        let pos2 = Point::new(-5.0, 0.0, 10.0);

        let matrix1 = orientation.to_transform_matrix(&pos1);
        let matrix2 = orientation.to_transform_matrix(&pos2);

        // Translation components should be different
        assert_eq!(matrix1[3], 1.0); // pos1.x
        assert_eq!(matrix1[7], 2.0); // pos1.y
        assert_eq!(matrix1[11], 3.0); // pos1.z

        assert_eq!(matrix2[3], -5.0); // pos2.x
        assert_eq!(matrix2[7], 0.0); // pos2.y
        assert_eq!(matrix2[11], 10.0); // pos2.z

        // Rotation components should be the same
        for i in [0, 1, 2, 4, 5, 6, 8, 9, 10] {
            assert_eq!(matrix1[i], matrix2[i]);
        }
    }
}
