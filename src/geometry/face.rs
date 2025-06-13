//! Triangular faces of the geodesic polyhedron.

use crate::geometry::Point;

/// A triangular face of the geodesic polyhedron.
///
/// Faces represent the triangular elements created during icosahedron subdivision.
/// Each face is defined by three vertices and has a unique ID for identification.
/// The centroids of faces become the boundary points of tiles in the dual polyhedron.
///
/// # Role in Geodesic Construction
///
/// 1. **Initial faces**: 20 triangular faces of the icosahedron
/// 2. **Subdivision**: Each face is recursively divided into smaller triangles
/// 3. **Projection**: All face vertices are projected onto the sphere surface
/// 4. **Dual creation**: Face centroids become tile boundary points
///
/// # Examples
///
/// ```rust
/// # use geotiles::{Face, Point};
/// let point1 = Point::new(0.0, 0.0, 0.0);
/// let point2 = Point::new(1.0, 0.0, 0.0);
/// let point3 = Point::new(0.0, 1.0, 0.0);
/// let mut face = Face::new(0, point1, point2, point3);
///
/// // Check if two faces share an edge
/// # let face1 = Face::new(0, Point::new(0.0, 0.0, 0.0), Point::new(1.0, 0.0, 0.0), Point::new(0.0, 1.0, 0.0));
/// # let face2 = Face::new(1, Point::new(0.0, 0.0, 0.0), Point::new(1.0, 0.0, 0.0), Point::new(1.0, 1.0, 0.0));
/// if face1.is_adjacent_to(&face2) {
///     println!("Faces share an edge");
/// }
///
/// // Get the centroid for tile boundary calculation
/// let centroid = face.get_centroid();
/// ```
#[derive(Debug, Clone)]
pub struct Face {
    /// Unique identifier for this face
    pub id: usize,
    /// The three vertices that define this triangular face
    pub points: [Point; 3],
    /// Cached centroid calculation (computed on first access)
    centroid: Option<Point>,
}

impl Face {
    /// Creates a new triangular face with the specified vertices.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this face
    /// * `p1`, `p2`, `p3` - The three vertices of the triangle
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::{Face, Point};
    /// let face = Face::new(
    ///     0,
    ///     Point::new(0.0, 0.0, 0.0),
    ///     Point::new(1.0, 0.0, 0.0),
    ///     Point::new(0.0, 1.0, 0.0)
    /// );
    /// ```
    pub fn new(id: usize, p1: Point, p2: Point, p3: Point) -> Self {
        Self {
            id,
            points: [p1, p2, p3],
            centroid: None,
        }
    }

    /// Returns the two vertices of this face that are not the specified point.
    ///
    /// This method is used during tile construction to find neighboring tiles.
    /// When processing a vertex that will become a tile center, this method
    /// identifies the other vertices of each surrounding face.
    ///
    /// # Arguments
    ///
    /// * `point` - The vertex to exclude from the result
    ///
    /// # Returns
    ///
    /// A vector containing the other two vertices of the face
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::{Face, Point};
    /// # let face = Face::new(0, Point::new(0.0, 0.0, 0.0), Point::new(1.0, 0.0, 0.0), Point::new(0.0, 1.0, 0.0));
    /// # let vertex = Point::new(0.0, 0.0, 0.0);
    /// let others = face.get_other_points(&vertex);
    /// assert_eq!(others.len(), 2); // Always returns exactly 2 points
    /// ```
    pub fn get_other_points(&self, point: &Point) -> Vec<&Point> {
        self.points.iter().filter(|p| *p != point).collect()
    }

    /// Finds the third vertex of the face given two known vertices.
    ///
    /// This is useful when you know two vertices of a face and need to find
    /// the third one, for example when traversing edges or reconstructing
    /// face connectivity.
    ///
    /// # Arguments
    ///
    /// * `p1`, `p2` - Two known vertices of the face
    ///
    /// # Returns
    ///
    /// The third vertex, or `None` if the two given points are not both vertices of this face
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::{Face, Point};
    /// # let face = Face::new(0, Point::new(0.0, 0.0, 0.0), Point::new(1.0, 0.0, 0.0), Point::new(0.0, 1.0, 0.0));
    /// # let point1 = Point::new(0.0, 0.0, 0.0);
    /// # let point2 = Point::new(1.0, 0.0, 0.0);
    /// if let Some(third_point) = face.find_third_point(&point1, &point2) {
    ///     println!("Found the third vertex: {}", third_point);
    /// }
    /// ```
    pub fn find_third_point(&self, p1: &Point, p2: &Point) -> Option<&Point> {
        self.points.iter().find(|p| *p != p1 && *p != p2)
    }

    /// Determines if this face is adjacent to another face (shares an edge).
    ///
    /// Two faces are considered adjacent if they share exactly two vertices.
    /// This relationship is used to determine tile neighborhoods and ensure
    /// proper connectivity in the geodesic structure.
    ///
    /// # Arguments
    ///
    /// * `face2` - The other face to check adjacency with
    ///
    /// # Returns
    ///
    /// `true` if the faces share exactly two vertices (one edge), `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::{Face, Point};
    /// # let face1 = Face::new(0, Point::new(0.0, 0.0, 0.0), Point::new(1.0, 0.0, 0.0), Point::new(0.0, 1.0, 0.0));
    /// # let face2 = Face::new(1, Point::new(0.0, 0.0, 0.0), Point::new(1.0, 0.0, 0.0), Point::new(1.0, 1.0, 0.0));
    /// if face1.is_adjacent_to(&face2) {
    ///     println!("Faces {} and {} share an edge", face1.id, face2.id);
    /// }
    /// ```
    pub fn is_adjacent_to(&self, face2: &Face) -> bool {
        let mut count = 0;
        for p1 in &self.points {
            for p2 in &face2.points {
                if p1 == p2 {
                    count += 1;
                }
            }
        }
        count == 2
    }

    /// Calculates and returns the centroid (geometric center) of the face.
    ///
    /// The centroid is the average of the three vertex positions. It's cached
    /// after the first calculation for efficiency. In the geodesic construction,
    /// face centroids become the boundary points of tiles.
    ///
    /// # Returns
    ///
    /// A reference to the centroid point
    ///
    /// # Mathematical Formula
    ///
    /// For vertices A, B, C:
    /// Centroid = (A + B + C) / 3
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::{Face, Point};
    /// # let mut face = Face::new(0, Point::new(0.0, 0.0, 0.0), Point::new(1.0, 0.0, 0.0), Point::new(0.0, 1.0, 0.0));
    /// let centroid = face.get_centroid();
    /// // The centroid is equidistant from all three vertices
    /// ```
    pub fn get_centroid(&mut self) -> &Point {
        if self.centroid.is_none() {
            let x = (self.points[0].x + self.points[1].x + self.points[2].x) / 3.0;
            let y = (self.points[0].y + self.points[1].y + self.points[2].y) / 3.0;
            let z = (self.points[0].z + self.points[1].z + self.points[2].z) / 3.0;

            self.centroid = Some(Point::new(x, y, z));
        }
        self.centroid.as_ref().unwrap()
    }
}
