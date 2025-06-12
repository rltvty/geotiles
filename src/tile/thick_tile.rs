//! 3D thick tile implementation with extrusion capabilities.

use super::tile::Tile;
use crate::geometry::{Point, Vector3};

/// A thick 3D tile with both inner and outer surfaces.
///
/// This struct represents a tile that has been extruded to create thickness,
/// suitable for 3D visualization, physics simulation, or manufacturing applications.
/// The thickness is applied uniformly inward from the original sphere surface,
/// maintaining the same center point but creating an inner boundary.
///
/// # Structure
///
/// - **Outer boundary**: Original tile boundary on the sphere surface
/// - **Inner boundary**: Extruded boundary points moved inward by the thickness
/// - **Side walls**: Connecting the outer and inner boundaries
///
/// # Applications
///
/// - 3D printed geodesic domes with wall thickness
/// - Game objects with collision volumes
/// - Architectural visualization
/// - Physical simulation of sphere-like structures
///
/// # Examples
///
/// ```rust
/// let thick_tiles = hexasphere.create_thick_tiles(0.5);
/// for thick_tile in thick_tiles {
///     let mesh_data = thick_tile.generate_all_vertices();
///     // mesh_data contains complete 3D geometry for rendering
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ThickTile {
    /// Vertices of the outer face (on the original sphere surface)
    pub outer_boundary: Vec<Point>,
    /// Vertices of the inner face (extruded inward by thickness)
    pub inner_boundary: Vec<Point>,
    /// Center point of the tile (on the outer surface)
    pub center_point: Point,
    /// Thickness of the tile (distance between outer and inner surfaces)
    pub thickness: f64,
    /// Whether this tile has 6 sides (hexagon) or 5 sides (pentagon)
    pub is_hexagon: bool,
}

impl ThickTile {
    /// Creates a thick tile by extruding a surface tile inward.
    ///
    /// The extrusion is performed by moving each boundary point inward along
    /// the surface normal by the specified thickness. The surface normal is
    /// calculated as the normalized vector from the sphere center to the tile center.
    ///
    /// # Arguments
    ///
    /// * `surface_tile` - The original 2D tile to extrude
    /// * `thickness` - How far to extrude inward (in same units as sphere radius)
    ///
    /// # Returns
    ///
    /// A new `ThickTile` with both outer and inner boundaries
    ///
    /// # Mathematical Details
    ///
    /// For each boundary point P on the sphere surface:
    /// 1. Calculate surface normal N = normalize(tile_center)
    /// 2. Inner point = P - N × thickness
    ///
    /// This ensures uniform thickness perpendicular to the sphere surface.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let surface_tile = &hexasphere.tiles[0];
    /// let thick_tile = ThickTile::from_surface_tile(surface_tile, 0.2);
    /// assert_eq!(thick_tile.thickness, 0.2);
    /// assert_eq!(thick_tile.outer_boundary.len(), thick_tile.inner_boundary.len());
    /// ```
    pub fn from_surface_tile(surface_tile: &Tile, thickness: f64) -> Self {
        let normal = Vector3::new(
            surface_tile.center_point.x,
            surface_tile.center_point.y,
            surface_tile.center_point.z,
        )
        .normalize();

        let inner_boundary = surface_tile
            .boundary
            .iter()
            .map(|point| {
                Point::new(
                    point.x - normal.x * thickness,
                    point.y - normal.y * thickness,
                    point.z - normal.z * thickness,
                )
            })
            .collect();

        Self {
            outer_boundary: surface_tile.boundary.clone(),
            inner_boundary,
            center_point: surface_tile.center_point.clone(),
            thickness,
            is_hexagon: surface_tile.is_hexagon(),
        }
    }

    /// Generates complete mesh data for the thick tile including all faces and sides.
    ///
    /// Creates a fully enclosed 3D mesh with:
    /// - **Outer face**: Triangulated from the center point outward
    /// - **Inner face**: Triangulated from the inner center point inward  
    /// - **Side walls**: Quadrilateral faces connecting outer and inner boundaries
    ///
    /// The resulting mesh has proper winding order for correct lighting:
    /// - Outer face: Counter-clockwise (normal pointing outward)
    /// - Inner face: Clockwise (normal pointing inward)
    /// - Side faces: Counter-clockwise from outside view
    ///
    /// # Returns
    ///
    /// A `ThickTileVertices` struct containing:
    /// - `vertices`: All 3D points in the mesh
    /// - `indices`: Triangle indices for rendering (groups of 3)
    ///
    /// # Mesh Structure
    ///
    /// The vertex array contains:
    /// 1. Outer center point (index 0)
    /// 2. Outer boundary points (indices 1 to N)
    /// 3. Inner center point (index N+1)
    /// 4. Inner boundary points (indices N+2 to 2N+1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// let mesh_data = thick_tile.generate_all_vertices();
    ///
    /// // Use with a 3D rendering library
    /// for triangle in mesh_data.indices.chunks(3) {
    ///     let v0 = &mesh_data.vertices[triangle[0]];
    ///     let v1 = &mesh_data.vertices[triangle[1]];
    ///     let v2 = &mesh_data.vertices[triangle[2]];
    ///     // Render triangle v0-v1-v2
    /// }
    /// ```
    pub fn generate_all_vertices(&self) -> ThickTileVertices {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut vertex_count = 0;

        // Add outer face vertices (as triangle fan from center)
        vertices.push(self.center_point.clone()); // Center vertex
        vertex_count += 1;

        for point in &self.outer_boundary {
            vertices.push(point.clone());
        }
        let outer_boundary_start = vertices.len();
        vertex_count += self.outer_boundary.len();

        // Create outer face triangles
        for i in 0..self.outer_boundary.len() {
            let next_i = (i + 1) % self.outer_boundary.len();
            indices.extend_from_slice(&[
                0, // Center
                outer_boundary_start + i,
                outer_boundary_start + next_i,
            ]);
        }

        // Add inner face vertices
        let inner_center = Point::new(
            self.center_point.x - self.get_normal().x * self.thickness,
            self.center_point.y - self.get_normal().y * self.thickness,
            self.center_point.z - self.get_normal().z * self.thickness,
        );

        vertices.push(inner_center);
        let inner_center_idx = vertex_count;
        vertex_count += 1;

        for point in &self.inner_boundary {
            vertices.push(point.clone());
        }
        let inner_boundary_start = vertex_count;
        vertex_count += self.inner_boundary.len();

        // Create inner face triangles (reversed winding for inward-facing normal)
        for i in 0..self.inner_boundary.len() {
            let next_i = (i + 1) % self.inner_boundary.len();
            indices.extend_from_slice(&[
                inner_center_idx,              // Center
                inner_boundary_start + next_i, // Reversed order
                inner_boundary_start + i,
            ]);
        }

        // Create side faces (quads as two triangles each)
        for i in 0..self.outer_boundary.len() {
            let next_i = (i + 1) % self.outer_boundary.len();

            let outer_curr = outer_boundary_start + i;
            let outer_next = outer_boundary_start + next_i;
            let inner_curr = inner_boundary_start + i;
            let inner_next = inner_boundary_start + next_i;

            // First triangle of quad
            indices.extend_from_slice(&[outer_curr, inner_curr, outer_next]);
            // Second triangle of quad
            indices.extend_from_slice(&[outer_next, inner_curr, inner_next]);
        }

        ThickTileVertices { vertices, indices }
    }

    /// Calculates the surface normal vector for this tile.
    ///
    /// For a tile on a sphere centered at the origin, the surface normal
    /// is simply the normalized vector from the origin to the tile center.
    /// This vector points directly outward from the sphere surface.
    ///
    /// # Returns
    ///
    /// A unit vector pointing outward from the sphere surface at this tile
    ///
    /// # Examples
    ///
    /// ```rust
    /// let normal = thick_tile.get_normal();
    /// let magnitude = (normal.x.powi(2) + normal.y.powi(2) + normal.z.powi(2)).sqrt();
    /// assert!((magnitude - 1.0).abs() < 0.001); // Should be unit vector
    /// ```
    fn get_normal(&self) -> Vector3 {
        Vector3::new(
            self.center_point.x,
            self.center_point.y,
            self.center_point.z,
        )
        .normalize()
    }

    /// Generates vertices for just the side walls of the thick tile.
    ///
    /// This method provides an alternative to `generate_all_vertices()` when you
    /// want to handle the top and bottom faces separately (e.g., for different
    /// materials or textures). The vertices are arranged to make quad generation easy.
    ///
    /// # Returns
    ///
    /// A vector of points arranged as: [outer₀, inner₀, outer₁, inner₁, ...]
    /// This interleaved format makes it easy to create quads by taking consecutive pairs.
    ///
    /// # Quad Generation
    ///
    /// For each edge i, you can create a quad using vertices:
    /// - `vertices[2*i]` (outer current)
    /// - `vertices[2*i + 1]` (inner current)  
    /// - `vertices[2*(i+1) + 1]` (inner next)
    /// - `vertices[2*(i+1)]` (outer next)
    ///
    /// # Examples
    ///
    /// ```rust
    /// let side_vertices = thick_tile.generate_side_vertices();
    ///
    /// // Create quads for side walls
    /// for i in 0..thick_tile.outer_boundary.len() {
    ///     let next_i = (i + 1) % thick_tile.outer_boundary.len();
    ///     
    ///     let outer_curr = &side_vertices[2 * i];
    ///     let inner_curr = &side_vertices[2 * i + 1];
    ///     let inner_next = &side_vertices[2 * next_i + 1];
    ///     let outer_next = &side_vertices[2 * next_i];
    ///     
    ///     // Create quad: outer_curr -> inner_curr -> inner_next -> outer_next
    /// }
    /// ```
    pub fn generate_side_vertices(&self) -> Vec<Point> {
        let mut vertices = Vec::new();

        // Interleave outer and inner boundary points for easy quad generation
        for i in 0..self.outer_boundary.len() {
            vertices.push(self.outer_boundary[i].clone());
            vertices.push(self.inner_boundary[i].clone());
        }

        vertices
    }
}

/// Complete vertex and index data for a thick tile mesh.
///
/// This struct contains all the geometric data needed to render a thick tile
/// as a 3D mesh. The data is formatted for use with standard 3D graphics APIs
/// and engines.
///
/// # Data Format
///
/// - **Vertices**: Array of 3D points representing all mesh vertices
/// - **Indices**: Array of vertex indices grouped into triangles (every 3 indices = 1 triangle)
///
/// # Usage with Graphics APIs
///
/// The data can be directly used with:
/// - **OpenGL**: Vertex Buffer Objects (VBOs) and Element Buffer Objects (EBOs)
/// - **Vulkan**: Vertex and index buffers
/// - **DirectX**: Vertex and index buffers  
/// - **WebGL**: Buffer data for vertex and index arrays
/// - **Game Engines**: Mesh construction in Unity, Unreal, Bevy, etc.
///
/// # Examples
///
/// ```rust
/// let mesh_data = thick_tile.generate_all_vertices();
///
/// // Convert to your graphics library's format
/// let positions: Vec<[f32; 3]> = mesh_data.vertices
///     .iter()
///     .map(|p| [p.x as f32, p.y as f32, p.z as f32])
///     .collect();
///     
/// let triangles: Vec<u32> = mesh_data.indices
///     .iter()
///     .map(|&i| i as u32)
///     .collect();
/// ```
#[derive(Debug, Clone)]
pub struct ThickTileVertices {
    /// All vertices in the mesh as 3D points
    pub vertices: Vec<Point>,
    /// Triangle indices (every 3 consecutive indices form one triangle)
    pub indices: Vec<usize>,
}
