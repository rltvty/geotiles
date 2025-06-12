//! Mathematical helper functions.

use crate::geometry::{Face, Point, Vector3};
use std::collections::HashMap;

// Helper functions

/// Calculates the surface normal of a triangle defined by three points.
///
/// Uses the cross product of two edge vectors to compute a vector perpendicular
/// to the triangle's surface. The direction of the normal follows the right-hand
/// rule based on the order of the input points.
///
/// # Arguments
///
/// * `p1` - First vertex of the triangle
/// * `p2` - Second vertex of the triangle  
/// * `p3` - Third vertex of the triangle
///
/// # Returns
///
/// A `Point` representing the surface normal vector (not normalized)
///
/// # Mathematical Details
///
/// Given triangle vertices A, B, C:
/// 1. Calculate edge vectors: U = B - A, V = C - A
/// 2. Compute cross product: N = U × V
/// 3. Return N (magnitude indicates triangle area × 2)
///
/// The cross product formula:
/// - N.x = U.y × V.z - U.z × V.y
/// - N.y = U.z × V.x - U.x × V.z
/// - N.z = U.x × V.y - U.y × V.x
///
/// # Winding Order
///
/// - **Counter-clockwise vertices**: Normal points toward viewer (outward)
/// - **Clockwise vertices**: Normal points away from viewer (inward)
/// - **Degenerate triangle**: Returns zero or near-zero vector
///
/// # Use Cases
///
/// - **Surface orientation**: Determining which way a face is pointing
/// - **Lighting calculations**: Surface normal for shading
/// - **Culling**: Back-face detection for rendering optimization
/// - **Tile boundary fixing**: Ensuring consistent winding order
///
/// # Examples
///
/// ```rust
/// // Counter-clockwise triangle (normal points up)
/// let p1 = Point::new(0.0, 0.0, 0.0);
/// let p2 = Point::new(1.0, 0.0, 0.0);
/// let p3 = Point::new(0.0, 1.0, 0.0);
/// let normal = calculate_surface_normal(&p1, &p2, &p3);
/// // normal.z > 0 (points toward +Z)
///
/// // Clockwise triangle (normal points down)
/// let normal_cw = calculate_surface_normal(&p1, &p3, &p2);
/// // normal_cw.z < 0 (points toward -Z)
/// ```
///
/// # Performance
///
/// - Time complexity: O(1) - constant time calculation
/// - Space complexity: O(1) - only creates temporary vectors
/// - Very fast: Just basic arithmetic operations
pub fn calculate_surface_normal(p1: &Point, p2: &Point, p3: &Point) -> Point {
    let u = Point::new(p2.x - p1.x, p2.y - p1.y, p2.z - p1.z);
    let v = Point::new(p3.x - p1.x, p3.y - p1.y, p3.z - p1.z);

    Point::new(
        u.y * v.z - u.z * v.y,
        u.z * v.x - u.x * v.z,
        u.x * v.y - u.y * v.x,
    )
}

/// Checks if a vector points away from the origin relative to a reference point.
///
/// Determines whether a vector is pointing "outward" from the sphere surface
/// by checking if the vector and point have the same general direction from
/// the origin. Used for ensuring tile boundaries have consistent outward orientation.
///
/// # Arguments
///
/// * `point` - Reference point (typically the tile center)
/// * `vector` - Direction vector to check (typically a surface normal)
///
/// # Returns
///
/// `true` if the vector points away from the origin, `false` otherwise
///
/// # Algorithm
///
/// For each coordinate axis, checks if the point and vector have the same sign:
/// - If point.x and vector.x are both positive or both negative: ✓
/// - If they have opposite signs: ✗
/// - Returns `true` only if ALL three axes agree
///
/// # Geometric Interpretation
///
/// - **True**: Vector points "outward" from the sphere center
/// - **False**: Vector points "inward" toward the sphere center
/// - **Edge case**: Returns `false` if any component has opposite signs
///
/// # Use Cases
///
/// - **Tile orientation**: Ensuring boundary normals point outward
/// - **Winding order correction**: Fixing reversed triangle orientations
/// - **Surface normal validation**: Checking calculated normals are correct
/// - **Rendering consistency**: Maintaining proper face orientation
///
/// # Limitations
///
/// This is a simplified heuristic that works well for points on a sphere
/// centered at the origin. For more complex geometries, a proper dot product
/// test would be more accurate: `dot(normalize(point), normalize(vector)) > 0`
///
/// # Examples
///
/// ```rust
/// // Point on sphere surface pointing outward
/// let center = Point::new(5.0, 5.0, 5.0);  // On sphere surface
/// let outward = Point::new(1.0, 1.0, 1.0); // Same direction as center
/// assert_eq!(pointing_away_from_origin(&center, &outward), true);
///
/// // Vector pointing inward
/// let inward = Point::new(-1.0, -1.0, -1.0); // Opposite direction
/// assert_eq!(pointing_away_from_origin(&center, &inward), false);
///
/// // Mixed directions (fails test)
/// let mixed = Point::new(1.0, -1.0, 1.0);   // Some components opposite
/// assert_eq!(pointing_away_from_origin(&center, &mixed), false);
/// ```
pub fn pointing_away_from_origin(point: &Point, vector: &Point) -> bool {
    (point.x * vector.x) >= 0.0 && (point.y * vector.y) >= 0.0 && (point.z * vector.z) >= 0.0
}

/// Subdivides a triangular face into smaller triangular faces recursively.
///
/// This is the core subdivision algorithm that transforms a single triangle into
/// multiple smaller triangles, creating the detailed geodesic structure. The
/// subdivision follows a regular pattern that maintains the triangle's shape
/// while increasing detail level.
///
/// # Arguments
///
/// * `face` - The triangular face to subdivide
/// * `num_divisions` - Number of subdivision levels (0 = no subdivision)
/// * `points` - HashMap for point deduplication and reuse
/// * `face_id` - Mutable reference to track face IDs for new faces
///
/// # Returns
///
/// A vector of `Face` objects representing all the smaller triangular faces
///
/// # Subdivision Pattern
///
/// For `num_divisions = n`, each triangle edge is divided into `n` segments,
/// creating a triangular grid pattern:
///
/// ```text
/// num_divisions = 0:    num_divisions = 1:    num_divisions = 2:
///      /\                    /\                    /\
///     /  \                  /  \                  /  \
///    /____\                /____\                /____\
///                         /\    /\              /\    /\
///                        /  \  /  \            /  \  /  \
///                       /____\/____\          /____\/____\
///                                            /\    /\    /\
///                                           /  \  /  \  /  \
///                                          /____\/____\/____\
/// ```
///
/// # Algorithm Steps
///
/// 1. **Edge subdivision**: Divide two edges of the triangle into segments
/// 2. **Row generation**: Create horizontal rows of points across the triangle
/// 3. **Triangle creation**: Form small triangles between adjacent rows
/// 4. **Point deduplication**: Reuse existing points from the HashMap
///
/// # Face Count Growth
///
/// - `num_divisions = 0`: 1 face (original triangle)
/// - `num_divisions = 1`: 4 faces
/// - `num_divisions = 2`: 16 faces
/// - `num_divisions = n`: 4^n faces (exponential growth)
///
/// # Point Management
///
/// The function uses a shared point HashMap to ensure that vertices are reused
/// between adjacent faces, preventing duplicate points and ensuring proper
/// connectivity in the final mesh.
///
/// # Use Cases
///
/// - **Geodesic generation**: Creating detailed sphere approximations
/// - **Level-of-detail**: Different subdivision levels for different uses
/// - **Mesh refinement**: Increasing triangle density for smoother surfaces
/// - **Icosahedron processing**: Applied to each of the 20 initial faces
///
/// # Examples
///
/// ```rust
/// use geotiles::Face;
/// use geotiles::Point;
/// use geotiles::utils::subdivide_face;
/// use std::collections::HashMap;
///
/// let mut points = HashMap::new();
/// let mut face_id = 0;
///
/// // Create a simple triangle
/// let face = Face::new(0,
///     Point::new(0.0, 0.0, 0.0),
///     Point::new(1.0, 0.0, 0.0),
///     Point::new(0.5, 1.0, 0.0)
/// );
///
/// // Subdivide once (1 → 4 triangles)
/// let subdivided = subdivide_face(face, 1, &mut points, &mut face_id);
/// assert_eq!(subdivided.len(), 4);
///
/// // Subdivide twice (1 → 16 triangles)
/// let face2 = Face::new(1, /* ... */);
/// let subdivided2 = subdivide_face(face2, 2, &mut points, &mut face_id);
/// assert_eq!(subdivided2.len(), 16);
/// ```
///
/// # Performance
///
/// - Time complexity: O(4^n) where n = num_divisions
/// - Space complexity: O(4^n) for face storage
/// - Memory usage grows exponentially with subdivision level
/// - Consider caching results for repeated use with same parameters
pub fn subdivide_face(
    face: Face,
    num_divisions: usize,
    points: &mut HashMap<Point, Point>,
    face_id: &mut usize,
) -> Vec<Face> {
    let mut new_faces = Vec::new();

    let left = subdivide_edge(&face.points[0], &face.points[1], num_divisions, points);
    let right = subdivide_edge(&face.points[0], &face.points[2], num_divisions, points);

    let mut prev_row = vec![face.points[0].clone()];

    for i in 1..=num_divisions {
        let current_row = subdivide_edge(&left[i], &right[i], i, points);

        // Create faces between rows
        for j in 0..i {
            let new_face = Face::new(
                *face_id,
                prev_row[j].clone(),
                current_row[j].clone(),
                current_row[j + 1].clone(),
            );
            *face_id += 1;
            new_faces.push(new_face);

            if j > 0 {
                let new_face = Face::new(
                    *face_id,
                    prev_row[j - 1].clone(),
                    prev_row[j].clone(),
                    current_row[j].clone(),
                );
                *face_id += 1;
                new_faces.push(new_face);
            }
        }

        prev_row = current_row;
    }

    new_faces
}

/// Subdivides an edge into multiple segments with intermediate points.
///
/// Creates evenly spaced points along a line segment between two endpoints.
/// This is a fundamental operation used during triangle subdivision to create
/// the vertex grid pattern. Points are managed through a HashMap to ensure
/// deduplication across multiple edge subdivisions.
///
/// # Arguments
///
/// * `p1` - Starting point of the edge
/// * `p2` - Ending point of the edge
/// * `count` - Number of segments to create (intermediate points + 1)
/// * `points` - HashMap for point deduplication and storage
///
/// # Returns
///
/// A vector of `Point` objects representing the subdivided edge, including
/// the original endpoints
///
/// # Point Distribution
///
/// For `count = n`, creates `n + 1` points total:
/// - Point 0: `p1` (start)
/// - Point 1: `p1 + 1/n * (p2 - p1)`
/// - Point 2: `p1 + 2/n * (p2 - p1)`
/// - ...
/// - Point n: `p2` (end)
///
/// # Mathematical Formula
///
/// Each intermediate point at position `i` is calculated as:
/// ```
/// point_i = p1 * (1 - t) + p2 * t
/// where t = i / count
/// ```
///
/// This is linear interpolation (lerp) between the two endpoints.
///
/// # Point Management
///
/// - **Deduplication**: Uses `get_or_insert_point()` to reuse existing points
/// - **Shared vertices**: Ensures edge endpoints are shared between adjacent faces
/// - **Memory efficiency**: Prevents duplicate points in the final mesh
/// - **Connectivity**: Maintains proper topology in the subdivided structure
///
/// # Use Cases
///
/// - **Triangle subdivision**: Creating intermediate vertices along triangle edges
/// - **Mesh refinement**: Increasing vertex density for smoother curves
/// - **Grid generation**: Creating regular point distributions
/// - **Geodesic construction**: Building the detailed vertex structure
///
/// # Examples
///
/// ```rust
/// use geotiles::Point;
/// use geotiles::utils::subdivide_edge;
/// use std::collections::HashMap;
///
/// let mut points = HashMap::new();
///
/// let start = Point::new(0.0, 0.0, 0.0);
/// let end = Point::new(3.0, 0.0, 0.0);
///
/// // Subdivide into 3 segments (4 points total)
/// let subdivided = subdivide_edge(&start, &end, 3, &mut points);
///
/// assert_eq!(subdivided.len(), 4);
/// assert_eq!(subdivided[0], start);              // 0.0
/// // subdivided[1] approximately (1.0, 0.0, 0.0)  // 1/3 of the way
/// // subdivided[2] approximately (2.0, 0.0, 0.0)  // 2/3 of the way  
/// assert_eq!(subdivided[3], end);                // 3.0
///
/// // Points are deduplicated in the HashMap
/// assert_eq!(points.len(), 4); // Only unique points stored
/// ```
///
/// # Performance
///
/// - Time complexity: O(n) where n = count
/// - Space complexity: O(n) for the result vector
/// - HashMap operations: O(1) average for point lookup/insertion
/// - Memory efficient due to point reuse
pub fn subdivide_edge(
    p1: &Point,
    p2: &Point,
    count: usize,
    points: &mut HashMap<Point, Point>,
) -> Vec<Point> {
    let mut result = Vec::new();
    result.push(get_or_insert_point(p1.clone(), points));

    for i in 1..count {
        let t = i as f64 / count as f64;
        let new_point = Point::new(
            p1.x * (1.0 - t) + p2.x * t,
            p1.y * (1.0 - t) + p2.y * t,
            p1.z * (1.0 - t) + p2.z * t,
        );
        result.push(get_or_insert_point(new_point, points));
    }

    result.push(get_or_insert_point(p2.clone(), points));
    result
}

/// Retrieves an existing point from the HashMap or inserts it if not present.
///
/// This function implements point deduplication by checking if a point with
/// identical coordinates already exists in the HashMap. If found, returns
/// the existing point; if not, inserts the new point and returns it.
///
/// # Arguments
///
/// * `point` - The point to retrieve or insert
/// * `points` - Mutable HashMap storing unique points
///
/// # Returns
///
/// A `Point` that is guaranteed to be stored in the HashMap
///
/// # Deduplication Strategy
///
/// Points are considered identical if their string representations match
/// (which includes the 3-decimal-place rounding from `Point::new()`). This
/// ensures that:
/// - Vertices shared between faces are truly shared (same memory location)
/// - No duplicate vertices exist in the final mesh
/// - Topology is properly maintained
///
/// # HashMap Behavior
///
/// - **Key**: The Point itself (using its Hash implementation)
/// - **Value**: The same Point (allows retrieval of the canonical instance)
/// - **Lookup**: O(1) average time complexity
/// - **Insertion**: O(1) average time complexity
///
/// # Use Cases
///
/// - **Vertex deduplication**: Ensuring unique vertices in mesh generation
/// - **Topology preservation**: Maintaining proper edge/face connectivity
/// - **Memory optimization**: Reducing redundant point storage
/// - **Mesh validation**: Guaranteeing valid mesh structure
///
/// # Why This Matters
///
/// Without point deduplication:
/// - Meshes would have duplicate vertices at the same locations
/// - Topology would be broken (faces wouldn't properly connect)
/// - Memory usage would be much higher
/// - Rendering and physics would have artifacts
///
/// # Examples
///
/// ```rust
/// let mut points = HashMap::new();
///
/// let p1 = Point::new(1.0, 2.0, 3.0);
/// let p2 = Point::new(1.0, 2.0, 3.0); // Same coordinates
///
/// // First insertion
/// let stored_p1 = get_or_insert_point(p1, &mut points);
/// assert_eq!(points.len(), 1);
///
/// // Second "insertion" returns existing point
/// let stored_p2 = get_or_insert_point(p2, &mut points);
/// assert_eq!(points.len(), 1); // Still only 1 unique point
///
/// // Both return the same canonical point
/// assert_eq!(stored_p1, stored_p2);
/// ```
///
/// # Performance
///
/// - Time complexity: O(1) average, O(n) worst case (hash collision)
/// - Space complexity: O(1) per unique point
/// - Hash quality: Depends on Point's Hash implementation
/// - Memory: Slight overhead for HashMap structure
pub fn get_or_insert_point(point: Point, points: &mut HashMap<Point, Point>) -> Point {
    if let Some(existing) = points.get(&point) {
        existing.clone()
    } else {
        points.insert(point.clone(), point.clone());
        point
    }
}

/// Finds the projected version of an original point in the projected points HashMap.
///
/// This function matches points from the pre-projection coordinate system with
/// their corresponding points in the post-projection (sphere surface) coordinate
/// system. It's used during the transition from the subdivided icosahedron to
/// the final spherical geodesic polyhedron.
///
/// # Arguments
///
/// * `original` - A point from the subdivided icosahedron (before sphere projection)
/// * `projected_points` - HashMap containing points after sphere projection
///
/// # Returns
///
/// `Some(Point)` if a matching projected point is found, `None` otherwise
///
/// # Matching Algorithm
///
/// Since direct coordinate matching won't work (projection changes coordinates),
/// this function compares normalized direction vectors:
///
/// 1. **Normalize original**: Convert to unit vector from origin
/// 2. **Check each projected point**: Convert to unit vector from origin  
/// 3. **Compare directions**: Calculate Euclidean distance between unit vectors
/// 4. **Threshold match**: If distance < 0.001, consider it a match
///
/// # Why This Is Needed
///
/// During hexasphere construction:
/// 1. Icosahedron vertices are subdivided (creating many points)
/// 2. All points are projected onto sphere surface (changing coordinates)
/// 3. Faces need to be grouped by their vertices to create tiles
/// 4. This function links original vertices to their projected locations
///
/// # Tolerance and Precision
///
/// - **Tolerance**: 0.001 difference in normalized coordinates
/// - **Precision**: Based on Point's 3-decimal-place rounding
/// - **Robustness**: Should handle floating-point precision errors
/// - **Edge cases**: May fail for points very close to origin
///
/// # Limitations
///
/// This is acknowledged as a "simplified version" with potential improvements:
/// - Could use more sophisticated matching for edge cases
/// - Might need tighter or adaptive tolerance
/// - Could benefit from spatial indexing for large point sets
/// - May have issues with degenerate or near-zero points
///
/// # Use Cases
///
/// - **Topology preservation**: Maintaining face-vertex relationships across projection
/// - **Tile construction**: Grouping faces by their projected vertices
/// - **Coordinate system bridging**: Linking pre- and post-projection geometry
/// - **Mesh validation**: Ensuring all vertices have projected counterparts
///
/// # Examples
///
/// ```rust
/// let mut projected_points = HashMap::new();
///
/// // Original point (before projection)
/// let original = Point::new(5.0, 5.0, 5.0);
///
/// // Its projected version (on unit sphere)
/// let mut projected = original.clone();
/// projected.project(1.0, 1.0); // Project to unit sphere
/// projected_points.insert(projected.clone(), projected.clone());
///
/// // Find the match
/// let found = find_projected_point(&original, &projected_points);
/// assert!(found.is_some());
///
/// // The found point should be on the sphere surface
/// let found_point = found.unwrap();
/// let distance_from_origin = (found_point.x.powi(2) + found_point.y.powi(2) + found_point.z.powi(2)).sqrt();
/// assert!((distance_from_origin - 1.0).abs() < 0.001); // Should be on unit sphere
/// ```
///
/// # Performance
///
/// - Time complexity: O(n) where n = number of projected points (linear search)
/// - Space complexity: O(1) additional memory
/// - Could be optimized with spatial indexing for large datasets
/// - Performance degrades with high subdivision levels
pub fn find_projected_point(
    original: &Point,
    projected_points: &HashMap<Point, Point>,
) -> Option<Point> {
    // This is a simplified version - in practice you might need more sophisticated matching
    for (projected, _) in projected_points {
        // Check if this could be the projected version by comparing normalized directions
        let orig_mag = (original.x.powi(2) + original.y.powi(2) + original.z.powi(2)).sqrt();
        let orig_norm = Point::new(
            original.x / orig_mag,
            original.y / orig_mag,
            original.z / orig_mag,
        );

        let proj_mag = (projected.x.powi(2) + projected.y.powi(2) + projected.z.powi(2)).sqrt();
        let proj_norm = Point::new(
            projected.x / proj_mag,
            projected.y / proj_mag,
            projected.z / proj_mag,
        );

        let diff = ((orig_norm.x - proj_norm.x).powi(2)
            + (orig_norm.y - proj_norm.y).powi(2)
            + (orig_norm.z - proj_norm.z).powi(2))
        .sqrt();

        if diff < 0.001 {
            return Some(projected.clone());
        }
    }
    None
}

/// Sorts faces around a point to ensure proper adjacency order.
///
/// This function is intended to arrange faces in the correct order around a
/// central vertex so that adjacent faces in the array share edges. However,
/// the current implementation is simplified and doesn't perform actual sorting.
///
/// # Arguments
///
/// * `faces` - Mutable slice of faces to sort around the point
/// * `_point` - The central point around which faces should be ordered (currently unused)
///
/// # Current Implementation
///
/// **Note**: This is a placeholder implementation that doesn't actually sort.
/// The faces remain in their original order. A full implementation would:
///
/// 1. **Find adjacencies**: Determine which faces share edges with each other
/// 2. **Build ordering**: Create a circular arrangement where adjacent faces share edges
/// 3. **Handle degeneracies**: Deal with edge cases and non-manifold geometry
/// 4. **Preserve winding**: Maintain consistent orientation around the point
///
/// # Why Sorting Is Important
///
/// Proper face ordering around a vertex is crucial for:
/// - **Tile boundary construction**: Creating properly ordered polygon boundaries
/// - **Normal calculation**: Ensuring consistent surface orientation
/// - **Rendering**: Proper triangle strip or fan generation
/// - **Topology validation**: Verifying manifold mesh properties
///
/// # Expected Algorithm (Future Implementation)
///
/// A complete implementation might:
///
/// ```rust
/// fn sort_faces_around_point(faces: &mut [Face], point: &Point) {
///     // 1. Calculate angles or use edge adjacency to determine order
///     // 2. Sort faces by angle around the central point
///     // 3. Handle degenerate cases (overlapping faces, etc.)
///     // 4. Ensure the resulting order forms a proper fan/strip
/// }
/// ```
///
/// # Impact of Simplified Version
///
/// The current simplified version may cause:
/// - **Incorrect tile boundaries**: Polygon points in wrong order
/// - **Winding issues**: Inconsistent face orientation
/// - **Visual artifacts**: Incorrect normals or lighting
/// - **Topology errors**: Non-manifold mesh structure
///
/// # Use Cases (When Properly Implemented)
///
/// - **Tile construction**: Ensuring polygon boundaries are correctly ordered
/// - **Mesh generation**: Creating valid triangle fans around vertices
/// - **Normal calculation**: Proper surface orientation computation
/// - **Manifold validation**: Checking mesh topology correctness
///
/// # Examples (Conceptual)
///
/// ```rust
/// let mut faces = vec![face1, face2, face3, face4, face5];
/// let center_point = Point::new(0.0, 0.0, 0.0);
///
/// // Currently: no-op, faces remain in original order
/// sort_faces_around_point(&mut faces, &center_point);
///
/// // Desired: faces would be reordered so adjacent faces share edges
/// // faces[0] shares edge with faces[1]
/// // faces[1] shares edge with faces[2]
/// // ...
/// // faces[4] shares edge with faces[0] (completing the loop)
/// ```
///
/// # Performance (When Implemented)
///
/// - Time complexity: O(n log n) for sorting, or O(n²) for adjacency-based ordering
/// - Space complexity: O(n) for temporary data structures
/// - Geometric calculations: Angle computation or edge comparison overhead
pub fn sort_faces_around_point(faces: &mut [Face], _point: &Point) {
    // This is a simplified version - the original JS has more complex ordering logic
    // For now, we'll keep the faces in their current order
    // A full implementation would sort faces to be adjacent around the point
}

/// Calculates the area of a triangle defined by three points using cross product.
///
/// Computes the surface area of a triangle in 3D space using the geometric
/// property that the magnitude of the cross product of two edge vectors equals
/// twice the triangle's area.
///
/// # Arguments
///
/// * `p1` - First vertex of the triangle
/// * `p2` - Second vertex of the triangle
/// * `p3` - Third vertex of the triangle
///
/// # Returns
///
/// The area of the triangle as a positive floating-point number
///
/// # Mathematical Background
///
/// Given triangle vertices A, B, C:
/// 1. **Calculate edge vectors**: v1 = B - A, v2 = C - A
/// 2. **Compute cross product**: cross = v1 × v2
/// 3. **Calculate magnitude**: |cross| = √(cross.x² + cross.y² + cross.z²)
/// 4. **Triangle area**: Area = |cross| / 2
///
/// # Why Cross Product Works
///
/// The cross product v1 × v2 produces a vector whose magnitude equals the area
/// of the parallelogram formed by v1 and v2. Since a triangle is half of this
/// parallelogram, we divide by 2 to get the triangle area.
///
/// # Properties
///
/// - **Always positive**: Returns absolute area regardless of vertex order
/// - **Units**: Result has units of distance² (same as input coordinates)
/// - **Degenerate triangles**: Returns 0 for collinear points
/// - **Precision**: Subject to floating-point precision limitations
///
/// # Use Cases
///
/// - **Tile area calculation**: Computing surface area of geodesic tiles
/// - **Mesh analysis**: Measuring triangle sizes for quality assessment
/// - **Statistical analysis**: Total surface area calculations
/// - **Validation**: Detecting degenerate triangles (near-zero area)
///
/// # Comparison with Other Methods
///
/// Alternative triangle area formulas:
/// - **Heron's formula**: Uses side lengths, more computation
/// - **Determinant method**: 2D version using coordinate determinant
/// - **Cross product**: Most efficient for 3D coordinates (this method)
///
/// # Examples
///
/// ```rust
/// // Right triangle with legs of length 3 and 4
/// use geotiles::Point;
/// use geotiles::utils::triangle_area;
///
/// let p1 = Point::new(0.0, 0.0, 0.0);
/// let p2 = Point::new(3.0, 0.0, 0.0);
/// let p3 = Point::new(0.0, 4.0, 0.0);
///
/// let area = triangle_area(&p1, &p2, &p3);
/// assert!((area - 6.0).abs() < 0.001); // Area = (1/2) × base × height = (1/2) × 3 × 4 = 6
///
/// // Degenerate triangle (collinear points)
/// let p4 = Point::new(6.0, 0.0, 0.0); // On same line as p1 and p2
/// let degenerate_area = triangle_area(&p1, &p2, &p4);
/// assert!(degenerate_area < 0.001); // Should be approximately 0
///
/// // Triangle in 3D space
/// let p5 = Point::new(1.0, 1.0, 1.0);
/// let p6 = Point::new(2.0, 1.0, 1.0);
/// let p7 = Point::new(1.0, 2.0, 1.0);
/// let area_3d = triangle_area(&p5, &p6, &p7);
/// assert!((area_3d - 0.5).abs() < 0.001); // Unit triangle area = 0.5
/// ```
///
/// # Performance
///
/// - Time complexity: O(1) - constant time calculation
/// - Space complexity: O(1) - only temporary vectors created
/// - Operations: 12 multiplications, 9 additions, 1 square root
/// - Very efficient for repeated area calculations
pub fn triangle_area(p1: &Point, p2: &Point, p3: &Point) -> f64 {
    // Using cross product to calculate triangle area
    let v1 = Vector3::new(p2.x - p1.x, p2.y - p1.y, p2.z - p1.z);
    let v2 = Vector3::new(p3.x - p1.x, p3.y - p1.y, p3.z - p1.z);
    let cross = v1.cross(&v2);
    0.5 * (cross.x.powi(2) + cross.y.powi(2) + cross.z.powi(2)).sqrt()
}
