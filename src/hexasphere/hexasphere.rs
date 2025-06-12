//! Core hexasphere implementation and construction.

use crate::approximation::RegularHexagonParams;
use crate::geometry::{Face, Point};
use crate::tile::{ThickTile, Tile, TileOrientation};
use crate::utils::{find_projected_point, sort_faces_around_point, subdivide_face};
use std::collections::HashMap;

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
/// use geotiles::Hexasphere;
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
    /// # Arguments
    ///
    /// * `radius` - Radius of the target sphere (determines overall size)
    /// * `num_divisions` - Number of subdivision levels (detail/complexity)
    ///   - 0: Just the icosahedron (12 tiles)
    ///   - 1: 42 tiles
    ///   - 2: 162 tiles  
    ///   - 3: 642 tiles
    ///   - 4: 2562 tiles
    ///   - n: ~10×4^(n-1) tiles (exponential growth)
    /// * `hex_size` - Scale factor for tile boundaries (0.01 to 1.0)
    ///   - 1.0: Tiles touch at boundaries (no gaps)
    ///   - 0.9: Small gaps between tiles (10% shrinkage)
    ///   - 0.5: Large gaps between tiles (50% shrinkage)
    ///
    /// # Performance Considerations
    ///
    /// Construction time grows exponentially with `num_divisions`:
    /// - 0-2: Nearly instant (< 1ms)
    /// - 3-4: Fast (< 100ms)
    /// - 5-6: Moderate (< 1s)
    /// - 7+: Slow (seconds to minutes)
    ///
    /// Memory usage also grows exponentially. Consider caching results for
    /// repeated use with the same parameters.
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
    /// // Small sphere for testing
    /// let small = Hexasphere::new(1.0, 2, 1.0);
    ///
    /// // Medium detail for visualization
    /// let medium = Hexasphere::new(10.0, 4, 0.9);
    ///
    /// // High detail for scientific applications
    /// let detailed = Hexasphere::new(100.0, 6, 0.95);
    ///
    /// // Debug version with gaps
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

        let mut faces: Vec<Face> = face_indices
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

        // Group faces by their points to create tiles
        let mut point_to_faces: HashMap<Point, Vec<usize>> = HashMap::new();
        for (face_idx, face) in new_faces.iter().enumerate() {
            for point in &face.points {
                // Find the projected version of this point
                if let Some(projected_point) = find_projected_point(point, &projected_points) {
                    point_to_faces
                        .entry(projected_point.clone())
                        .or_insert_with(Vec::new)
                        .push(face_idx);
                }
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
    ///     spawn_regular_hexagon_mesh(transform, hex_params.radius);
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
    ///             spawn_uniform_hexagon(transform, uniform_radius);
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
    ///                 spawn_hexagon_mesh(transform);
    ///             } else {
    ///                 spawn_pentagon_mesh(transform);
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
    /// let hex_orientations = hexasphere.get_hexagon_orientations();
    /// let uniform_radius = hexasphere.get_uniform_hexagon_radius();
    ///
    /// println!("Processing {} hexagonal tiles", hex_orientations.len());
    ///
    /// for (i, orientation) in hex_orientations.iter().enumerate() {
    ///     // Note: 'i' here is NOT the tile index in hexasphere.tiles
    ///     let transform = orientation.to_transform_matrix(&Point::new(0.0, 0.0, 0.0)); // placeholder center
    ///     spawn_regular_hexagon_mesh(transform, uniform_radius);
    /// }
    ///
    /// // If you need tile correspondence, use this instead:
    /// for (tile_index, tile) in hexasphere.tiles.iter().enumerate() {
    ///     if tile.is_hexagon() {
    ///         if let Some(orientation) = tile.get_orientation() {
    ///             let transform = orientation.to_transform_matrix(&tile.center_point);
    ///             spawn_hexagon_with_tile_id(transform, uniform_radius, tile_index);
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
    /// let outer_sphere = Hexasphere::new(10.0, 4, 0.9);
    /// let inner_sphere = outer_sphere.create_inner_sphere(9.0);
    ///
    /// assert_eq!(outer_sphere.tiles.len(), inner_sphere.tiles.len());
    /// assert_eq!(inner_sphere.radius, 9.0);
    ///
    /// // Connect corresponding tiles
    /// for (outer_tile, inner_tile) in outer_sphere.tiles.iter().zip(inner_sphere.tiles.iter()) {
    ///     // Create connecting geometry between outer and inner boundaries
    ///     create_connecting_walls(&outer_tile.boundary, &inner_tile.boundary);
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
    ///     create_3d_mesh_from_data(mesh_data);
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
}
