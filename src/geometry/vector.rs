//! 3D vector operations for coordinate systems and transformations.

/// A 3D vector with normalization and cross product operations.
///
/// Used for calculating orientations, surface normals, and coordinate system transformations.
/// Unlike `Point`, this represents a direction and magnitude rather than a position.
///
/// # Examples
///
/// ```rust
/// let v1 = Vector3::new(1.0, 0.0, 0.0);
/// let v2 = Vector3::new(0.0, 1.0, 0.0);
/// let cross = v1.cross(&v2); // Should point in Z direction
/// let normalized = v1.normalize(); // Unit vector in X direction
/// ```
#[derive(Debug, Clone)]
pub struct Vector3 {
    /// X component of the vector
    pub x: f64,
    /// Y component of the vector
    pub y: f64,
    /// Z component of the vector
    pub z: f64,
}

impl Vector3 {
    /// Creates a new 3D vector with the specified components.
    ///
    /// # Arguments
    ///
    /// * `x` - X component
    /// * `y` - Y component
    /// * `z` - Z component
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Returns a normalized (unit) vector in the same direction.
    ///
    /// A normalized vector has a magnitude of 1.0 while preserving direction.
    /// If the vector has zero magnitude, returns a zero vector.
    ///
    /// # Returns
    ///
    /// A new `Vector3` with magnitude 1.0 (or zero if input was zero)
    ///
    /// # Examples
    ///
    /// ```rust
    /// let v = Vector3::new(3.0, 4.0, 0.0);
    /// let unit = v.normalize();
    /// assert!((unit.x - 0.6).abs() < 0.001); // 3/5
    /// assert!((unit.y - 0.8).abs() < 0.001); // 4/5
    /// ```
    pub fn normalize(&self) -> Self {
        let mag = (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt();
        if mag == 0.0 {
            Self::new(0.0, 0.0, 0.0)
        } else {
            Self::new(self.x / mag, self.y / mag, self.z / mag)
        }
    }

    /// Calculates the cross product with another vector.
    ///
    /// The cross product produces a vector perpendicular to both input vectors,
    /// with magnitude equal to the area of the parallelogram formed by the vectors.
    /// The direction follows the right-hand rule.
    ///
    /// # Arguments
    ///
    /// * `other` - The second vector for the cross product
    ///
    /// # Returns
    ///
    /// A new `Vector3` representing the cross product
    ///
    /// # Mathematical Formula
    ///
    /// For vectors A×B:
    /// - x = A.y × B.z - A.z × B.y
    /// - y = A.z × B.x - A.x × B.z  
    /// - z = A.x × B.y - A.y × B.x
    ///
    /// # Examples
    ///
    /// ```rust
    /// let x_axis = Vector3::new(1.0, 0.0, 0.0);
    /// let y_axis = Vector3::new(0.0, 1.0, 0.0);
    /// let z_axis = x_axis.cross(&y_axis);
    /// assert!((z_axis.z - 1.0).abs() < 0.001); // Should point in +Z direction
    /// ```
    pub fn cross(&self, other: &Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// Calculates the dot product with another vector.
    ///
    /// The dot product is the sum of the products of corresponding components.
    /// Geometrically, it equals |A||B|cos(θ) where θ is the angle between vectors.
    ///
    /// # Arguments
    ///
    /// * `other` - The second vector for the dot product
    ///
    /// # Returns
    ///
    /// The dot product as a scalar value
    ///
    /// # Examples
    ///
    /// ```rust
    /// let v1 = Vector3::new(1.0, 2.0, 3.0);
    /// let v2 = Vector3::new(4.0, 5.0, 6.0);
    /// let dot = v1.dot(&v2); // 1*4 + 2*5 + 3*6 = 32
    /// assert_eq!(dot, 32.0);
    /// ```
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}
