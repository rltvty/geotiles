//! Tile orientation and coordinate system calculations.

use crate::geometry::{Vector3, Point};

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
    /// let matrix = orientation.to_rotation_matrix();
    /// // Use with graphics libraries that expect 3x3 rotation matrices
    /// ```
    pub fn to_rotation_matrix(&self) -> [f64; 9] {
        [
            self.right.x, self.up.x, self.forward.x,
            self.right.y, self.up.y, self.forward.y,
            self.right.z, self.up.z, self.forward.z,
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
    /// let transform = orientation.to_transform_matrix(&tile.center_point);
    /// // Use with 3D engines like Bevy, Three.js, etc.
    /// ```
    pub fn to_transform_matrix(&self, translation: &Point) -> [f64; 16] {
        [
            self.right.x, self.up.x, self.forward.x, translation.x,
            self.right.y, self.up.y, self.forward.y, translation.y,
            self.right.z, self.up.z, self.forward.z, translation.z,
            0.0, 0.0, 0.0, 1.0,
        ]
    }
}
