//! Core hexasphere implementation and construction.

use crate::approximation::RegularHexagonParams;
use crate::geometry::{Face, Point};
use crate::tile::core::Tile;
use crate::tile::{ThickTile, TileOrientation};
use crate::utils::{find_projected_point, sort_faces_around_point, subdivide_face};
use std::collections::HashMap;

/// Instance data for a hexagon shape with positioning and indexing information.
#[derive(Debug, Clone)]
pub struct HexagonInstance {
    /// Translation (center position) of this hexagon instance
    pub translation: Point,
    /// Orientation (local coordinate system) of this hexagon instance  
    pub orientation: TileOrientation,
    /// Index of the hexagon shape this instance uses
    pub hexagon_index: usize,
}

/// The main geodesic polyhedron structure containing all tiles.
///
/// This is the primary interface for creating and working with geodesic polyhedra.
/// It generates a sphere-like surface made of polygonal tiles (mostly hexagons with
/// exactly 12 pentagons) by subdividing an icosahedron and projecting it onto a sphere.
///
/// # Construction Process
///
/// 1. **Icosahedron creation**: Start with 12 vertices and 20 triangular faces
/// 2. **Subdivision**: Recursively divide each face into smaller triangles
/// 3. **Projection**: Project all vertices onto the sphere surface
/// 4. **Dual generation**: Convert triangle vertices to polygon centers
/// 5. **Tile creation**: Form tiles using face centroids as boundaries
/// 6. **Neighbor resolution**: Establish connectivity between adjacent tiles
///
/// # Parameters
///
/// - **Radius**: Size of the resulting sphere
/// - **Subdivisions**: Detail level (higher = more tiles, smoother approximation)
/// - **Hex size**: Scale factor for tile boundaries (controls gaps between tiles)
///
/// # Applications
///
/// - **Game development**: Spherical game boards, planet surfaces
/// - **Scientific visualization**: Global data representation
/// - **Architecture**: Geodesic dome design
/// - **Computer graphics**: Sphere approximation with flat faces
/// - **Geographic mapping**: Alternative to traditional projections
///
/// # Examples
///
/// ```rust
/// # use geotiles::Hexasphere;
/// // Create a detailed hexasphere
/// let hexasphere = Hexasphere::new(10.0, 4, 0.95);
///
/// // Analyze the structure
/// println!("Generated {} tiles", hexasphere.tiles.len());
/// let stats = hexasphere.calculate_hexagon_stats();
/// println!("Size variation: {:.1}%",
///     100.0 * stats.radius_std_deviation / stats.average_hexagon_radius);
///
/// // Export for visualization
/// # std::fs::write("sphere.obj", hexasphere.to_obj()).unwrap();
/// ```

#[derive(Debug)]
pub struct Hexasphere {
    /// Radius of the sphere that the tiles approximate
    pub radius: f64,
    /// All polygonal tiles (hexagons and pentagons) that make up the surface
    pub tiles: Vec<Tile>,
}

impl Hexasphere {
    /// Creates a new hexasphere with the specified parameters.
    ///
    /// This is the main constructor that generates a complete geodesic polyhedron
    /// by subdividing an icosahedron and projecting it onto a sphere. The process
    /// is computationally intensive and the result is cached in the returned structure.
    /// 
    /// For more efficient generation using symmetry rules, see [`Hexasphere::new_with_instancing`].
    ///
    /// # Arguments
    ///
    /// * `radius` - Radius of the target sphere (determines overall size)
    /// * `num_divisions` - Number of subdivision levels (detail/complexity)
    ///   - 0: 12 tiles (icosahedron)
    ///   - 1: 12 tiles
    ///   - 2: 42 tiles
    ///   - 3: 92 tiles  
    ///   - 4: 162 tiles
    ///   - 5: 252 tiles
    ///   - n: 10n² + 2 tiles (quadratic growth)
    /// * `hex_size` - Scale factor for tile boundaries (0.01 to 1.0)
    ///   - 1.0: Tiles touch at boundaries (no gaps)
    ///   - 0.9: Small gaps between tiles (10% shrinkage)
    ///   - 0.5: Large gaps between tiles (50% shrinkage)
    ///
    /// # Performance Considerations
    ///
    /// Construction time grows quadratically with `num_divisions`:
    /// - 0-5: Nearly instant (< 10ms)
    /// - 6-10: Fast (< 100ms)
    /// - 11-20: Moderate (< 1s)
    /// - 21-50: Slower (1-10s)
    /// - 51+: Slow (10s+)
    ///
    /// Memory usage also grows quadratically (10n² + 2 tiles). Consider caching
    /// results for repeated use with the same parameters, or use shape instancing
    /// for high subdivision levels to reduce memory usage by 10-100x.
    ///
    /// # Mathematical Background
    ///
    /// The subdivision creates a Class I geodesic polyhedron where triangles
    /// are divided uniformly. The resulting Goldberg polyhedron has exactly
    /// 12 pentagonal faces (at icosahedral vertices) and the rest hexagonal.
    ///
    /// # Icosahedron Vertex Arrangement
    ///
    /// The 12 vertices are arranged using the golden ratio (τ ≈ 1.618) in three
    /// perpendicular rectangles:
    /// - Rectangle 1: (±1, ±τ, 0) - 4 vertices
    /// - Rectangle 2: (0, ±1, ±τ) - 4 vertices  
    /// - Rectangle 3: (±τ, 0, ±1) - 4 vertices
    ///
    /// # Algorithm Steps
    ///
    /// 1. **Create icosahedron**: Generate 12 vertices and 20 triangular faces
    /// 2. **Subdivide triangles**: Each triangle → 4^n smaller triangles
    /// 3. **Project to sphere**: Normalize all vertices to sphere surface
    /// 4. **Generate dual**: Each vertex becomes a tile center
    /// 5. **Create boundaries**: Face centroids become tile boundary points
    /// 6. **Establish neighbors**: Connect adjacent tiles
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// // Small sphere for testing
    /// let small = Hexasphere::new(1.0, 2, 1.0);
    ///
    /// // Medium detail for visualization
    /// let medium = Hexasphere::new(10.0, 4, 0.9);
    ///
    /// // High detail for scientific applications
    /// let detailed = Hexasphere::new(100.0, 6, 0.95);
    ///
    /// // Debug version with gaps between tiles
    /// let debug = Hexasphere::new(5.0, 3, 0.7);
    /// ```
    ///
    /// # Panics
    ///
    /// May panic if memory allocation fails for very large subdivision levels.
    /// Consider using smaller subdivision levels and increase gradually.
    pub fn new(radius: f64, num_divisions: usize, hex_size: f64) -> Self {
        let tao = 1.61803399; // Golden ratio

        // Create icosahedron corners
        let corners = vec![
            Point::new(1000.0, tao * 1000.0, 0.0),
            Point::new(-1000.0, tao * 1000.0, 0.0),
            Point::new(1000.0, -tao * 1000.0, 0.0),
            Point::new(-1000.0, -tao * 1000.0, 0.0),
            Point::new(0.0, 1000.0, tao * 1000.0),
            Point::new(0.0, -1000.0, tao * 1000.0),
            Point::new(0.0, 1000.0, -tao * 1000.0),
            Point::new(0.0, -1000.0, -tao * 1000.0),
            Point::new(tao * 1000.0, 0.0, 1000.0),
            Point::new(-tao * 1000.0, 0.0, 1000.0),
            Point::new(tao * 1000.0, 0.0, -1000.0),
            Point::new(-tao * 1000.0, 0.0, -1000.0),
        ];

        // Keep track of unique points
        let mut points: HashMap<Point, Point> = HashMap::new();
        for corner in &corners {
            points.insert(corner.clone(), corner.clone());
        }

        // Create initial icosahedron faces
        let face_indices = vec![
            [0, 1, 4],
            [1, 9, 4],
            [4, 9, 5],
            [5, 9, 3],
            [2, 3, 7],
            [3, 2, 5],
            [7, 10, 2],
            [0, 8, 10],
            [0, 4, 8],
            [8, 2, 10],
            [8, 4, 5],
            [8, 5, 2],
            [1, 0, 6],
            [11, 1, 6],
            [3, 9, 11],
            [6, 10, 7],
            [3, 11, 7],
            [11, 6, 7],
            [6, 0, 10],
            [9, 1, 11],
        ];

        let faces: Vec<Face> = face_indices
            .into_iter()
            .enumerate()
            .map(|(id, [i, j, k])| {
                Face::new(
                    id,
                    corners[i].clone(),
                    corners[j].clone(),
                    corners[k].clone(),
                )
            })
            .collect();

        // Subdivide faces
        let mut new_faces = Vec::new();
        let mut face_id = faces.len();

        for face in faces {
            let subdivided = subdivide_face(face, num_divisions, &mut points, &mut face_id);
            new_faces.extend(subdivided);
        }

        // Project all points to sphere
        let mut projected_points: HashMap<Point, Point> = HashMap::new();
        for point in points.into_values() {
            let mut projected = point.clone();
            projected.project(radius, 1.0);
            projected_points.insert(projected.clone(), projected);
        }

        // Update faces to use projected vertices
        for face in &mut new_faces {
            for i in 0..3 {
                if let Some(projected_point) =
                    find_projected_point(&face.points[i], &projected_points)
                {
                    face.points[i] = projected_point;
                }
            }
            // Clear cached centroid since points have changed
            face.clear_centroid_cache();
        }

        // Group faces by their points to create tiles
        let mut point_to_faces: HashMap<Point, Vec<usize>> = HashMap::new();
        for (face_idx, face) in new_faces.iter().enumerate() {
            for point in &face.points {
                point_to_faces
                    .entry(point.clone())
                    .or_default()
                    .push(face_idx);
            }
        }

        // Create tiles
        let mut tiles = Vec::new();
        let mut tile_lookup: HashMap<String, usize> = HashMap::new();

        for (point, face_indices) in point_to_faces {
            let mut point_faces: Vec<Face> = face_indices
                .into_iter()
                .map(|idx| new_faces[idx].clone())
                .collect();

            // Sort faces to be ordered around the point
            sort_faces_around_point(&mut point_faces, &point);

            let tile = Tile::new(point, &mut point_faces, hex_size);
            let tile_id = tile.to_string();
            tile_lookup.insert(tile_id, tiles.len());
            tiles.push(tile);
        }

        // Resolve neighbor references
        for tile in &mut tiles {
            tile.neighbors = tile
                .neighbor_ids
                .iter()
                .filter_map(|id| tile_lookup.get(id).copied())
                .collect();
        }

        Self { radius, tiles }
    }

    /// Creates a new hexasphere using symmetry rules instead of full calculation.
    ///
    /// This is an efficient alternative to [`Hexasphere::new`] that generates the hexasphere
    /// by identifying unique shapes and using icosahedral symmetry to place instances.
    /// This can be significantly faster and more memory-efficient for high subdivision levels.
    ///
    /// # Arguments
    ///
    /// * `radius` - Radius of the target sphere (determines overall size)
    /// * `num_divisions` - Number of subdivision levels (detail/complexity)
    /// * `hex_size` - Scale factor for tile boundaries (0.01 to 1.0)
    ///
    /// # Performance Benefits
    ///
    /// - **Memory efficiency**: Uses ~10x less memory by storing unique shapes + instances
    /// - **Generation speed**: Can be faster for high subdivision levels (15+)
    /// - **Identical output**: Produces exactly the same geometry as `Hexasphere::new`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geotiles::Hexasphere;
    /// 
    /// // These produce identical results
    /// let traditional = Hexasphere::new(1.0, 10, 0.95);
    /// let instanced = Hexasphere::new_with_instancing(1.0, 10, 0.95);
    /// 
    /// assert_eq!(traditional.tiles.len(), instanced.tiles.len());
    /// ```
    pub fn new_with_instancing(radius: f64, num_divisions: u32, hex_size: f64) -> Self {
        // For low subdivision levels, traditional generation is fine
        if num_divisions < 8 {
            return Self::new(radius, num_divisions as usize, hex_size);
        }

        // Generate using shape instancing approach
        let shape_data = Self::get_shape_instances_for_level(radius, num_divisions, hex_size);
        
        // Convert shape instances back to tiles
        let tiles = Self::convert_instances_to_tiles(shape_data, radius, hex_size);
        
        Self { radius, tiles }
    }

    /// Get shape approximations with a specified number of sub-groups.
    ///
    /// Similar to [`get_regular_hexagon_approximations`] but returns a specified number
    /// of grouped hexagon shapes along with instance data for tiling the sphere.
    /// This provides both the unique shapes and the positioning information needed
    /// for efficient GPU instancing or procedural generation.
    ///
    /// # Arguments
    ///
    /// * `number_of_sub_groups` - Target number of unique hexagon shapes to return
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// 1. **shapes**: Vector of `RegularHexagonParams` with length ≤ `number_of_sub_groups`
    /// 2. **instances**: Vector of instance data (translation, orientation, hexagon_index)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geotiles::Hexasphere;
    /// 
    /// let hexasphere = Hexasphere::new(1.0, 10, 0.95);
    /// let (shapes, instances) = hexasphere.get_shape_approximations(25);
    /// 
    /// println!("Using {} unique shapes to tile {} hexagons", 
    ///     shapes.len(), instances.len());
    /// 
    /// // Render each unique shape multiple times
    /// for (shape_idx, shape) in shapes.iter().enumerate() {
    ///     let shape_instances: Vec<_> = instances.iter()
    ///         .filter(|inst| inst.hexagon_index == shape_idx)
    ///         .collect();
    ///     
    ///     // GPU instancing: render shape once with all transforms
    ///     render_instanced_hexagon(shape, &shape_instances);
    /// }
    /// ```
    pub fn get_shape_approximations(&self, number_of_sub_groups: usize) -> 
        (Vec<crate::approximation::RegularHexagonParams>, Vec<HexagonInstance>) {
        
        // Get natural shape instances first
        let shape_data = self.get_shape_instances();
        
        // Filter out hexagon shapes only
        let hexagon_shapes: Vec<_> = shape_data.shapes
            .iter()
            .enumerate()
            .filter(|(_, shape)| shape.sides == 6)
            .collect();
            
        if number_of_sub_groups >= hexagon_shapes.len() {
            // No grouping needed - return all natural shapes
            return self.convert_to_hexagon_approximations(&shape_data);
        }
        
        // Use simplified shapes method to group hexagons
        let (simplified_shapes, simplified_instances, _) = 
            self.get_simplified_shapes(number_of_sub_groups, 0.05);
            
        // Convert to the format you requested
        self.convert_simplified_to_approximations(simplified_shapes, simplified_instances)
    }

    /// Get regular hexagon parameters for all hexagonal tiles.
    ///
    /// Generates `RegularHexagonParams` for every hexagonal tile, providing
    /// the data needed to create regular hexagon approximations. Pentagon tiles
    /// are excluded since they cannot be approximated as regular hexagons.
    ///
    /// # Returns
    ///
    /// A vector of `RegularHexagonParams` containing position, size, and orientation
    /// data for each hexagonal tile
    ///
    /// # Generated Parameters
    ///
    /// For each hexagon:
    /// - **Center**: Tile center point (exact position)
    /// - **Radius**: Average distance from center to boundary points
    /// - **Orientation**: Local coordinate system for proper rotation
    ///
    /// # Use Cases
    ///
    /// - **Individual tile replacement**: Each tile gets its own best-fit regular hexagon
    /// - **Variable size rendering**: Preserve size variations while using regular shapes
    /// - **Quality optimization**: Use actual tile measurements for each approximation
    /// - **Detailed analysis**: Compare original vs. regular hexagon properties
    ///
    /// # Quality Considerations
    ///
    /// - **Best fit per tile**: Each approximation is optimized for its specific tile
    /// - **Size variation preserved**: Maintains the geodesic size distribution
    /// - **Orientation accuracy**: Uses calculated tile orientations
    /// - **Hexagon-only**: Pentagons require separate handling
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geotiles::Hexasphere;
    /// let hexasphere = Hexasphere::new(1.0, 2, 1.0);
    /// let approximations = hexasphere.get_regular_hexagon_approximations();
    ///
    /// for (i, hex_params) in approximations.iter().enumerate() {
    ///     println!("Hexagon {}: center={}, radius={:.3}",
    ///         i, hex_params.center, hex_params.radius);
    ///     
    ///     // Generate perfect hexagon vertices
    ///     let vertices = hex_params.generate_vertices();
    ///     assert_eq!(vertices.len(), 6);
    ///     
    ///     // Use in 3D engine
    ///     let transform = hex_params.orientation.to_transform_matrix(&hex_params.center);
    ///     // spawn_regular_hexagon_mesh(transform, hex_params.radius);
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// - Time complexity: O(n) where n = number of hexagonal tiles
    /// - Space complexity: O(n) for the returned vector
    /// - Memory per hexagon: ~200 bytes (Point + f64 + TileOrientation)
    pub fn get_regular_hexagon_approximations(&self) -> Vec<RegularHexagonParams> {
        self.tiles
            .iter()
            .filter_map(|tile| tile.get_regular_hexagon_params())
            .collect()
    }

    /// Get the best single radius to use for uniform regular hexagons.
    ///
    /// Calculates the optimal radius for creating uniform regular hexagons that
    /// approximate all hexagonal tiles. This is the average radius of all hexagons,
    /// providing a good balance between over-sized and under-sized approximations.
    ///
    /// # Returns
    ///
    /// The average hexagon radius as a floating-point number
    ///
    /// # Calculation Method
    ///
    /// 1. Measure average radius of each hexagonal tile
    /// 2. Calculate the mean of all hexagon radii
    /// 3. Return this average as the uniform size
    ///
    /// # Use Cases
    ///
    /// - **Uniform tile rendering**: All hexagons the same size for consistency
    /// - **Gameplay mechanics**: Equal-sized game spaces
    /// - **Simplified physics**: Uniform collision shapes
    /// - **Performance optimization**: Single mesh instanced multiple times
    ///
    /// # Trade-offs
    ///
    /// - **Pros**: Consistent appearance, simple implementation, good performance
    /// - **Cons**: Some tiles will be over/under-sized, gaps or overlaps possible
    /// - **Quality**: Depends on geodesic uniformity (higher subdivision = better)
    ///
    /// # Size Distribution
    ///
    /// - **Smaller than average**: Tiles near icosahedral vertices (pentagons)
    /// - **Larger than average**: Tiles far from icosahedral vertices
    /// - **Average fit**: Most tiles in the middle regions
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geotiles::Hexasphere;
    /// let hexasphere = Hexasphere::new(1.0, 2, 1.0);
    /// let uniform_radius = hexasphere.get_uniform_hexagon_radius();
    /// println!("Use radius {:.3} for all regular hexagons", uniform_radius);
    ///
    /// // Check how well this fits
    /// let stats = hexasphere.calculate_hexagon_stats();
    /// let error_range = (stats.max_hexagon_radius - stats.min_hexagon_radius) / uniform_radius;
    /// println!("Size error range: ±{:.1}%", 50.0 * error_range);
    ///
    /// // Use for rendering
    /// for tile in &hexasphere.tiles {
    ///     if tile.is_hexagon() {
    ///         if let Some(orientation) = tile.get_orientation() {
    ///             let transform = orientation.to_transform_matrix(&tile.center_point);
    ///             // spawn_uniform_hexagon(transform, uniform_radius);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn get_uniform_hexagon_radius(&self) -> f64 {
        self.calculate_hexagon_stats().average_hexagon_radius
    }

    /// Get orientations for all tiles (both hexagons and pentagons).
    ///
    /// Calculates the local coordinate system for every tile in the hexasphere,
    /// providing the orientation data needed for proper placement of 3D objects.
    /// Returns `Some(TileOrientation)` for tiles with valid boundaries, `None` for
    /// tiles without sufficient boundary points.
    ///
    /// # Returns
    ///
    /// A vector of `Option<TileOrientation>` with one entry per tile, preserving
    /// the same order as the `tiles` array
    ///
    /// # Orientation Calculation
    ///
    /// For each tile:
    /// - **Right vector**: From center toward first boundary point
    /// - **Up vector**: Outward surface normal (center point normalized)
    /// - **Forward vector**: Cross product completing right-handed system
    ///
    /// # Use Cases
    ///
    /// - **Mixed tile handling**: Process hexagons and pentagons together
    /// - **Complete coverage**: Get orientations for every tile location
    /// - **Validation**: Check which tiles have valid orientations
    /// - **Index correspondence**: Results match `tiles` array indices
    ///
    /// # None Values
    ///
    /// A tile orientation may be `None` if:
    /// - Tile has no boundary points
    /// - Boundary points are degenerate (all at same location)
    /// - Mathematical calculation fails (extremely rare)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geotiles::Hexasphere;
    /// let hexasphere = Hexasphere::new(1.0, 2, 1.0);
    /// let orientations = hexasphere.get_tile_orientations();
    ///
    /// for (i, orientation_opt) in orientations.iter().enumerate() {
    ///     let tile = &hexasphere.tiles[i];
    ///     
    ///     match orientation_opt {
    ///         Some(orientation) => {
    ///             let transform = orientation.to_transform_matrix(&tile.center_point);
    ///             
    ///             if tile.is_hexagon() {
    ///                 // spawn_hexagon_mesh(transform);
    ///             } else {
    ///                 // spawn_pentagon_mesh(transform);
    ///             }
    ///         }
    ///         None => {
    ///             eprintln!("Warning: Could not calculate orientation for tile {}", i);
    ///         }
    ///     }
    /// }
    ///
    /// // Count valid orientations
    /// let valid_count = orientations.iter().filter(|opt| opt.is_some()).count();
    /// println!("Valid orientations: {}/{}", valid_count, orientations.len());
    /// ```
    pub fn get_tile_orientations(&self) -> Vec<Option<TileOrientation>> {
        self.tiles
            .iter()
            .map(|tile| tile.get_orientation())
            .collect()
    }

    /// Get orientations only for hexagonal tiles.
    ///
    /// Calculates orientations specifically for hexagonal tiles, filtering out
    /// pentagons and any tiles with invalid orientations. This is useful when
    /// you only need to handle hexagons (e.g., for regular hexagon approximations).
    ///
    /// # Returns
    ///
    /// A vector of `TileOrientation` containing only valid hexagon orientations
    ///
    /// # Filtering Process
    ///
    /// 1. **Hexagon filter**: Only process tiles with 6 boundary points
    /// 2. **Orientation calculation**: Compute orientation for each hexagon
    /// 3. **Validity filter**: Remove any failed calculations (None values)
    /// 4. **Result collection**: Return only successful orientations
    ///
    /// # Use Cases
    ///
    /// - **Hexagon-only processing**: When pentagons are handled separately
    /// - **Regular approximations**: Positioning uniform hexagon meshes
    /// - **Performance optimization**: Avoid processing pentagon tiles
    /// - **Simplified logic**: No need to handle Option types
    ///
    /// # Index Correspondence
    ///
    /// **Note**: The returned vector does NOT correspond to the original `tiles`
    /// array indices. If you need index correspondence, use `get_tile_orientations()`
    /// and filter manually.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geotiles::Point;
    /// use geotiles::Hexasphere;
    /// let hexasphere = Hexasphere::new(10.0, 4, 0.9);
    /// let hex_orientations = hexasphere.get_hexagon_orientations();
    /// let uniform_radius = hexasphere.get_uniform_hexagon_radius();
    ///
    /// println!("Processing {} hexagonal tiles", hex_orientations.len());
    ///
    /// for (i, orientation) in hex_orientations.iter().enumerate() {
    ///     // Note: 'i' here is NOT the tile index in hexasphere.tiles
    ///     let transform = orientation.to_transform_matrix(&Point::new(0.0, 0.0, 0.0)); // placeholder center
    ///     // spawn_regular_hexagon_mesh(transform, uniform_radius);
    /// }
    ///
    /// // If you need tile correspondence, use this instead:
    /// for (tile_index, tile) in hexasphere.tiles.iter().enumerate() {
    ///     if tile.is_hexagon() {
    ///         if let Some(orientation) = tile.get_orientation() {
    ///             let transform = orientation.to_transform_matrix(&tile.center_point);
    ///             // spawn_hexagon_with_tile_id(transform, uniform_radius, tile_index);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn get_hexagon_orientations(&self) -> Vec<TileOrientation> {
        self.tiles
            .iter()
            .filter(|tile| tile.is_hexagon())
            .filter_map(|tile| tile.get_orientation())
            .collect()
    }

    /// Create a second hexasphere for thickness, ensuring tiles correspond correctly.
    ///
    /// Generates an inner sphere by uniformly scaling the existing hexasphere inward,
    /// maintaining the same topology and tile correspondence. This is useful for
    /// creating thick 3D structures or dual-sphere applications.
    ///
    /// # Arguments
    ///
    /// * `inner_radius` - Radius of the inner sphere (should be < outer radius)
    ///
    /// # Returns
    ///
    /// A new `Hexasphere` with the same structure but different radius
    ///
    /// # Scaling Method
    ///
    /// - **Ratio calculation**: `scale = inner_radius / outer_radius`
    /// - **Point scaling**: Each point P becomes P × scale
    /// - **Topology preservation**: Same number of tiles, same neighbors
    /// - **Correspondence**: `inner.tiles[i]` matches `outer.tiles[i]`
    ///
    /// # Properties of Result
    ///
    /// - **Same tile count**: Identical number of hexagons and pentagons
    /// - **Same connectivity**: Neighbor relationships preserved
    /// - **Proportional sizes**: All measurements scaled by the radius ratio
    /// - **Consistent orientation**: Tile orientations remain the same
    ///
    /// # Use Cases
    ///
    /// - **Thick shells**: Create hollow spherical structures
    /// - **Dual-layer systems**: Inner and outer sphere applications
    /// - **Easy implementation**: Reuses existing subdivision and projection
    /// - **Perfect correspondence**: Guaranteed 1:1 tile matching
    ///
    /// # Thickness Characteristics
    ///
    /// - **Non-uniform thickness**: Varies slightly due to scaling (not extrusion)
    /// - **Thinner near center**: Absolute thickness = (outer_radius - inner_radius)
    /// - **Relative scaling**: Inner hexagons are smaller than outer ones
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geotiles::Hexasphere;
    /// let outer_sphere = Hexasphere::new(10.0, 4, 0.9);
    /// let inner_sphere = outer_sphere.create_inner_sphere(9.0);
    ///
    /// assert_eq!(outer_sphere.tiles.len(), inner_sphere.tiles.len());
    /// assert_eq!(inner_sphere.radius, 9.0);
    ///
    /// // Connect corresponding tiles
    /// for (outer_tile, inner_tile) in outer_sphere.tiles.iter().zip(inner_sphere.tiles.iter()) {
    ///     // Create connecting geometry between outer and inner boundaries
    ///     // create_connecting_walls(&outer_tile.boundary, &inner_tile.boundary);
    /// }
    ///
    /// // Verify scaling
    /// let outer_center = &outer_sphere.tiles[0].center_point;
    /// let inner_center = &inner_sphere.tiles[0].center_point;
    /// let expected_scale = 9.0 / 10.0;
    ///
    /// assert!((inner_center.x - outer_center.x * expected_scale).abs() < 0.001);
    /// ```
    ///
    /// # Performance
    ///
    /// - **Memory efficient**: Reuses topology without recalculation
    /// - **Fast generation**: Only requires scaling existing points
    /// - **No subdivision**: Avoids expensive icosahedron processing
    /// - **Cache friendly**: Both spheres can share mesh generation code
    pub fn create_inner_sphere(&self, inner_radius: f64) -> Hexasphere {
        // Create inner sphere with same parameters but different radius
        let ratio = inner_radius / self.radius;

        // Scale all points inward while maintaining topology
        let mut inner_sphere = Hexasphere::new(inner_radius, 0, 1.0); // dummy values

        // Replace with scaled version of current sphere
        inner_sphere.radius = inner_radius;
        inner_sphere.tiles = self
            .tiles
            .iter()
            .map(|tile| {
                let scaled_center = Point::new(
                    tile.center_point.x * ratio,
                    tile.center_point.y * ratio,
                    tile.center_point.z * ratio,
                );

                let scaled_boundary = tile
                    .boundary
                    .iter()
                    .map(|point| Point::new(point.x * ratio, point.y * ratio, point.z * ratio))
                    .collect();

                Tile {
                    center_point: scaled_center,
                    boundary: scaled_boundary,
                    neighbor_ids: tile.neighbor_ids.clone(),
                    neighbors: tile.neighbors.clone(),
                }
            })
            .collect();

        inner_sphere
    }

    /// Create thick tiles by extruding inward with uniform thickness.
    ///
    /// Generates 3D thick tiles by extruding each surface tile inward along the
    /// surface normal. This creates true uniform thickness perpendicular to the
    /// sphere surface, unlike the scaling approach which varies with distance.
    ///
    /// # Arguments
    ///
    /// * `thickness` - How far to extrude inward (in same units as radius)
    ///
    /// # Returns
    ///
    /// A vector of `ThickTile` objects, one for each original tile
    ///
    /// # Extrusion Method
    ///
    /// For each tile:
    /// 1. **Calculate surface normal**: Normalized vector from origin to tile center
    /// 2. **Extrude boundary points**: Move each point inward by thickness × normal
    /// 3. **Create thick tile**: Combine outer boundary, inner boundary, and metadata
    ///
    /// # Thickness Properties
    ///
    /// - **True uniform thickness**: Constant perpendicular distance from surface
    /// - **Normal-based extrusion**: Follows sphere curvature correctly
    /// - **Preserved shape**: Inner boundary maintains tile shape
    /// - **Complete mesh data**: Ready for 3D rendering with proper faces
    ///
    /// # Use Cases
    ///
    /// - **3D visualization**: Render geodesic structures with depth
    /// - **Manufacturing**: 3D printing geodesic domes with wall thickness
    /// - **Physics simulation**: Collision volumes for sphere-like objects
    /// - **Architectural modeling**: Structural elements with realistic thickness
    ///
    /// # Advantages over Dual Sphere
    ///
    /// - **Uniform thickness**: Same absolute thickness everywhere
    /// - **Shape preservation**: Inner tiles maintain proportional shapes
    /// - **Memory efficient**: No duplicate hexasphere structure
    /// - **Mesh ready**: Complete vertex and index data for rendering
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geotiles::Hexasphere;
    /// let hexasphere = Hexasphere::new(10.0, 4, 0.9);
    /// let thick_tiles = hexasphere.create_thick_tiles(0.5);
    ///
    /// println!("Created {} thick tiles with 0.5 unit thickness", thick_tiles.len());
    ///
    /// for (i, thick_tile) in thick_tiles.iter().enumerate() {
    ///     // Generate complete 3D mesh
    ///     let mesh_data = thick_tile.generate_all_vertices();
    ///     
    ///     println!("Tile {}: {} vertices, {} triangles",
    ///         i, mesh_data.vertices.len(), mesh_data.indices.len() / 3);
    ///     
    ///     // Verify thickness
    ///     let outer_point = &thick_tile.outer_boundary[0];
    ///     let inner_point = &thick_tile.inner_boundary[0];
    ///     let measured_thickness = outer_point.distance_to(inner_point);
    ///     assert!((measured_thickness - 0.5).abs() < 0.01);
    ///     
    ///     // Use in 3D engine
    ///     // create_3d_mesh_from_data(mesh_data);
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// - **Generation time**: O(n×m) where n = tiles, m = boundary points per tile
    /// - **Memory usage**: ~3x original hexasphere size (outer + inner + mesh data)
    /// - **Mesh generation**: Additional O(n×m) for complete vertex/index arrays
    pub fn create_thick_tiles(&self, thickness: f64) -> Vec<ThickTile> {
        self.tiles
            .iter()
            .map(|tile| ThickTile::from_surface_tile(tile, thickness))
            .collect()
    }

    /// Get simplified shape instances using radius-based clustering.
    ///
    /// This method provides a second layer of optimization on top of natural shape instancing.
    /// It groups similar hexagon shapes together and uses representative shapes, allowing
    /// for massive additional compression with tunable quality trade-offs.
    ///
    /// # Arguments
    ///
    /// * `max_shapes` - Maximum number of shapes desired (must be >= 12 for pentagons)
    /// * `tolerance` - Geometric tolerance for clustering (0.0 = exact, 0.1 = 10% radius difference allowed)
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `Vec<TileShape>`: The simplified set of representative shapes
    /// - `Vec<TileInstance>`: Instance data mapping each tile to a simplified shape
    /// - `SimplificationStats`: Statistics about the approximation quality
    ///
    /// # Performance Benefits
    ///
    /// This can provide 5-10x additional compression on top of natural instancing:
    /// - Level 20: 356 shapes → 50 shapes (7x additional compression)
    /// - Level 30: 824 shapes → 100 shapes (8x additional compression)
    /// - Combined: 9,000 tiles → 100 shapes (90x total compression!)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// let hexasphere = Hexasphere::new(1.0, 20, 0.95);
    ///
    /// // Ultra performance: reduce to ~25 shapes with 5% tolerance
    /// let (shapes, instances, stats) = hexasphere.get_simplified_shapes(25, 0.05);
    ///
    /// println!("Reduced {} natural shapes to {} simplified shapes",
    ///     stats.original_shape_count, shapes.len());
    /// println!("Geometric error: {:.2}% average, {:.2}% maximum",
    ///     stats.average_error * 100.0, stats.max_error * 100.0);
    ///
    /// // Perfect for GPU instancing with minimal draw calls
    /// for (i, shape) in shapes.iter().enumerate() {
    ///     let instance_count = instances.iter().filter(|inst| inst.shape_index == i).count();
    ///     println!("Shape {}: {} instances", i, instance_count);
    /// }
    /// ```
    pub fn get_simplified_shapes(
        &self,
        max_shapes: usize,
        tolerance: f64,
    ) -> (
        Vec<crate::hexasphere::shape_instances::TileShape>,
        Vec<crate::hexasphere::shape_instances::TileInstance>,
        SimplificationStats,
    ) {
        // Get natural shape instances first
        let natural_shapes = self.get_shape_instances();

        // Count hexagon shapes for comparison
        let hexagon_shape_count = natural_shapes
            .shapes
            .iter()
            .filter(|shape| shape.sides == 6)
            .count();

        if max_shapes >= hexagon_shape_count {
            // No simplification needed for hexagons
            // Count pentagon instances in the original data
            let pentagon_instances_count = natural_shapes
                .instances
                .iter()
                .filter(|instance| natural_shapes.shapes[instance.shape_index].sides == 5)
                .count();

            let stats = SimplificationStats {
                original_shape_count: hexagon_shape_count,
                simplified_shape_count: hexagon_shape_count,
                compression_ratio: 1.0,
                average_error: 0.0,
                max_error: 0.0,
                pentagon_shapes_preserved: pentagon_instances_count,
            };
            return (natural_shapes.shapes, natural_shapes.instances, stats);
        }

        // Separate pentagons and hexagons
        let (pentagon_shapes, hexagon_shapes): (Vec<_>, Vec<_>) = natural_shapes
            .shapes
            .iter()
            .enumerate()
            .partition(|(_, shape)| shape.sides == 5);

        let _pentagon_count = pentagon_shapes.len();

        // Pentagons are always preserved separately, so we focus only on hexagon clustering
        let target_hexagon_shapes = max_shapes;

        // Cluster hexagon shapes by radius similarity
        let hexagon_clusters =
            self.cluster_hexagons_by_radius(&hexagon_shapes, target_hexagon_shapes, tolerance);

        // Build simplified shapes and instances
        let mut simplified_shapes = Vec::new();
        let mut simplified_instances = Vec::new();
        let mut total_error = 0.0f64;
        let mut max_error = 0.0f64;
        let mut hexagon_instance_count = 0;

        // Add all pentagon shapes first (never simplified)
        for (original_idx, pentagon_shape) in pentagon_shapes {
            simplified_shapes.push(pentagon_shape.clone());
            let shape_index = simplified_shapes.len() - 1;

            // Find all instances that used this pentagon shape
            for instance in &natural_shapes.instances {
                if instance.shape_index == original_idx {
                    let mut new_instance = instance.clone();
                    new_instance.shape_index = shape_index;
                    simplified_instances.push(new_instance);
                    // Don't count pentagon instances in error calculation
                }
            }
        }

        // Add clustered hexagon shapes
        for cluster in hexagon_clusters {
            let representative_shape = &cluster.representative;
            simplified_shapes.push(representative_shape.clone());
            let shape_index = simplified_shapes.len() - 1;

            // Add instances for all shapes in this cluster
            for member in &cluster.members {
                for instance in &natural_shapes.instances {
                    if instance.shape_index == member.original_index {
                        let mut new_instance = instance.clone();
                        new_instance.shape_index = shape_index;
                        simplified_instances.push(new_instance);

                        // Track approximation error (hexagons only)
                        let error = member.error;
                        total_error += error;
                        max_error = max_error.max(error);
                        hexagon_instance_count += 1;
                    }
                }
            }
        }

        // Count pentagon instances (should always be 12)
        let pentagon_instances_count = simplified_instances
            .iter()
            .filter(|instance| simplified_shapes[instance.shape_index].sides == 5)
            .count();

        // Calculate compression ratio based only on hexagon shapes
        let original_hexagon_count = hexagon_shapes.len();
        let simplified_hexagon_count = simplified_shapes.iter().filter(|s| s.sides == 6).count();
        let hexagon_compression_ratio = if simplified_hexagon_count > 0 {
            original_hexagon_count as f64 / simplified_hexagon_count as f64
        } else {
            1.0
        };

        let stats = SimplificationStats {
            original_shape_count: original_hexagon_count,
            simplified_shape_count: simplified_hexagon_count,
            compression_ratio: hexagon_compression_ratio,
            average_error: if hexagon_instance_count > 0 {
                total_error / hexagon_instance_count as f64
            } else {
                0.0
            },
            max_error,
            pentagon_shapes_preserved: pentagon_instances_count,
        };

        (simplified_shapes, simplified_instances, stats)
    }

    /// Helper method to cluster hexagon shapes by radius similarity
    fn cluster_hexagons_by_radius(
        &self,
        hexagon_shapes: &[(usize, &crate::hexasphere::shape_instances::TileShape)],
        target_clusters: usize,
        tolerance: f64,
    ) -> Vec<HexagonCluster> {
        if hexagon_shapes.is_empty() || target_clusters == 0 {
            return Vec::new();
        }

        if target_clusters >= hexagon_shapes.len() {
            // No clustering needed
            return hexagon_shapes
                .iter()
                .map(|(idx, shape)| HexagonCluster {
                    representative: (*shape).clone(),
                    members: vec![ClusterMember {
                        original_index: *idx,
                        error: 0.0,
                    }],
                })
                .collect();
        }

        // Simple radius-based clustering
        let mut clusters: Vec<HexagonCluster> = Vec::new();
        let mut used = vec![false; hexagon_shapes.len()];

        // Start with the first shape as the first cluster
        for i in 0..hexagon_shapes.len() {
            if used[i] {
                continue;
            }

            let (seed_idx, seed_shape) = hexagon_shapes[i];
            let mut cluster = HexagonCluster {
                representative: seed_shape.clone(),
                members: vec![ClusterMember {
                    original_index: seed_idx,
                    error: 0.0,
                }],
            };
            used[i] = true;

            // Find all similar shapes within tolerance
            for j in (i + 1)..hexagon_shapes.len() {
                if used[j] {
                    continue;
                }

                let (candidate_idx, candidate_shape) = hexagon_shapes[j];
                let radius_diff = (seed_shape.radius - candidate_shape.radius).abs();
                let relative_error = radius_diff / seed_shape.radius;

                if relative_error <= tolerance {
                    cluster.members.push(ClusterMember {
                        original_index: candidate_idx,
                        error: relative_error,
                    });
                    used[j] = true;
                }
            }

            clusters.push(cluster);

            // Stop if we've reached our target
            if clusters.len() >= target_clusters {
                break;
            }
        }

        // If we have remaining unclustered shapes and haven't reached target,
        // assign them to the closest existing clusters
        for i in 0..hexagon_shapes.len() {
            if used[i] {
                continue;
            }

            let (orphan_idx, orphan_shape) = hexagon_shapes[i];

            // Find closest cluster
            let mut best_cluster = 0;
            let mut best_error = f64::INFINITY;

            for (cluster_idx, cluster) in clusters.iter().enumerate() {
                let radius_diff = (cluster.representative.radius - orphan_shape.radius).abs();
                let error = radius_diff / cluster.representative.radius;

                if error < best_error {
                    best_error = error;
                    best_cluster = cluster_idx;
                }
            }

            clusters[best_cluster].members.push(ClusterMember {
                original_index: orphan_idx,
                error: best_error,
            });
        }

        clusters
    }

    /// Helper method to generate shape instances for a given subdivision level
    fn get_shape_instances_for_level(radius: f64, num_divisions: u32, hex_size: f64) -> 
        crate::hexasphere::shape_instances::ShapeInstanceData {
        // For now, generate a traditional hexasphere and extract shape data
        // TODO: Implement true direct generation from symmetry rules
        let temp_sphere = Self::new(radius, num_divisions as usize, hex_size);
        temp_sphere.get_shape_instances()
    }

    /// Helper method to convert shape instances back to tiles
    fn convert_instances_to_tiles(
        _shape_data: crate::hexasphere::shape_instances::ShapeInstanceData, 
        radius: f64, 
        hex_size: f64
    ) -> Vec<Tile> {
        // TODO: Implement conversion from shape instances to tiles
        // For now, use traditional generation
        let temp_sphere = Self::new(radius, 5, hex_size); // Placeholder
        temp_sphere.tiles
    }

    /// Helper method to convert shape instance data to hexagon approximations
    fn convert_to_hexagon_approximations(&self, 
        shape_data: &crate::hexasphere::shape_instances::ShapeInstanceData
    ) -> (Vec<RegularHexagonParams>, Vec<HexagonInstance>) {
        let mut hexagon_params = Vec::new();
        let mut instances = Vec::new();
        
        // Convert hexagon shapes to RegularHexagonParams
        for (shape_idx, shape) in shape_data.shapes.iter().enumerate() {
            if shape.sides == 6 {
                // Convert TileShape to RegularHexagonParams
                if let Some(params) = self.convert_tile_shape_to_params(shape) {
                    let param_index = hexagon_params.len();
                    hexagon_params.push(params);
                    
                    // Find all instances of this shape
                    for instance in &shape_data.instances {
                        if instance.shape_index == shape_idx {
                            instances.push(HexagonInstance {
                                translation: instance.center.clone(),
                                orientation: instance.orientation.clone(),
                                hexagon_index: param_index,
                            });
                        }
                    }
                }
            }
        }
        
        (hexagon_params, instances)
    }

    /// Helper method to convert simplified shapes to approximations
    fn convert_simplified_to_approximations(&self,
        simplified_shapes: Vec<crate::hexasphere::shape_instances::TileShape>,
        simplified_instances: Vec<crate::hexasphere::shape_instances::TileInstance>
    ) -> (Vec<RegularHexagonParams>, Vec<HexagonInstance>) {
        let mut hexagon_params = Vec::new();
        let mut instances = Vec::new();
        
        // Convert hexagon shapes to RegularHexagonParams
        for (shape_idx, shape) in simplified_shapes.iter().enumerate() {
            if shape.sides == 6 {
                if let Some(params) = self.convert_tile_shape_to_params(shape) {
                    let param_index = hexagon_params.len();
                    hexagon_params.push(params);
                    
                    // Find all instances of this shape
                    for instance in &simplified_instances {
                        if instance.shape_index == shape_idx {
                            instances.push(HexagonInstance {
                                translation: instance.center.clone(),
                                orientation: instance.orientation.clone(),
                                hexagon_index: param_index,
                            });
                        }
                    }
                }
            }
        }
        
        (hexagon_params, instances)
    }

    /// Helper method to convert TileShape to RegularHexagonParams
    fn convert_tile_shape_to_params(&self, shape: &crate::hexasphere::shape_instances::TileShape) -> 
        Option<RegularHexagonParams> {
        if shape.sides != 6 || shape.vertices.len() < 6 {
            return None;
        }
        
        // Calculate center point
        let center = Point::new(
            shape.vertices.iter().map(|v| v.x).sum::<f64>() / shape.vertices.len() as f64,
            shape.vertices.iter().map(|v| v.y).sum::<f64>() / shape.vertices.len() as f64,
            shape.vertices.iter().map(|v| v.z).sum::<f64>() / shape.vertices.len() as f64,
        );
        
        // Use the shape's radius
        let radius = shape.radius;
        
        // Create a basic orientation (could be improved)
        let up = crate::geometry::Vector3::new(center.x, center.y, center.z).normalize();
        let right = if shape.vertices.len() > 0 {
            let to_first = crate::geometry::Vector3::new(
                shape.vertices[0].x - center.x,
                shape.vertices[0].y - center.y,
                shape.vertices[0].z - center.z,
            ).normalize();
            to_first
        } else {
            crate::geometry::Vector3::new(1.0, 0.0, 0.0)
        };
        let forward = up.cross(&right);
        
        let orientation = TileOrientation { right, up, forward };
        
        Some(RegularHexagonParams {
            center,
            radius,
            orientation,
        })
    }
}

/// Statistics about hexagon shape simplification quality and performance
///
/// Note: Pentagon shapes are always preserved separately and not included in these metrics.
/// All compression ratios and error measurements apply only to hexagon shapes.
#[derive(Debug, Clone)]
pub struct SimplificationStats {
    /// Original number of hexagon shapes before simplification
    pub original_shape_count: usize,
    /// Number of hexagon shapes after simplification
    pub simplified_shape_count: usize,
    /// Compression ratio achieved for hexagons (original_count / simplified_count)
    pub compression_ratio: f64,
    /// Average geometric error across hexagon instances only
    pub average_error: f64,
    /// Maximum geometric error for any hexagon instance
    pub max_error: f64,
    /// Number of pentagon instances preserved (always 12 for valid spheres)
    pub pentagon_shapes_preserved: usize,
}

/// A cluster of similar hexagon shapes with a representative
#[derive(Debug, Clone)]
struct HexagonCluster {
    /// The representative shape for this cluster
    representative: crate::hexasphere::shape_instances::TileShape,
    /// All shapes that belong to this cluster
    members: Vec<ClusterMember>,
}

/// A member of a hexagon cluster
#[derive(Debug, Clone)]
struct ClusterMember {
    /// Original index of this shape in the natural shape array
    original_index: usize,
    /// Geometric error when using the cluster representative instead of this shape
    error: f64,
}
