//! # Geotiles - Geodesic Polyhedron Library
//!
//! This library generates geodesic polyhedra (specifically Goldberg polyhedra) by subdividing
//! an icosahedron and projecting the result onto a sphere. The resulting structure consists
//! mostly of hexagonal tiles with exactly 12 pentagonal tiles, creating a sphere-like surface
//! that approximates a true sphere with flat polygonal faces.
//!
//! ## Mathematical Background
//!
//! A geodesic polyhedron is constructed by:
//! 1. Starting with a regular icosahedron (20 triangular faces)
//! 2. Subdividing each triangular face into smaller triangles
//! 3. Projecting all vertices onto a sphere surface
//! 4. Creating tiles where each vertex becomes the center of a polygonal tile
//! 5. The tile boundaries are formed by connecting the centroids of surrounding triangular faces
//!
//! This process results in a Goldberg polyhedron, which is the dual of the geodesic polyhedron.
//! Due to Euler's formula for polyhedra (V - E + F = 2), it's impossible to tile a sphere
//! with only regular hexagons - exactly 12 pentagons are required.
//!
//! ## Key Concepts
//!
//! - **Vertex**: A point in 3D space, initially from the icosahedron corners
//! - **Face**: A triangular surface formed by three vertices
//! - **Tile**: A polygonal region (hexagon or pentagon) centered at a vertex
//! - **Boundary**: The edges of a tile, formed by connecting face centroids
//! - **Neighbors**: Adjacent tiles that share an edge
//!
//! ## Usage Example
//!
//! ```rust
//! use geotiles::Hexasphere;
//!
//! // Create a hexasphere with radius 10, 3 subdivision levels, 90% tile size
//! let hexasphere = Hexasphere::new(10.0, 3, 0.9);
//!
//! println!("Generated {} tiles", hexasphere.tiles.len());
//! 
//! // Analyze hexagon properties for regular hexagon approximation
//! let stats = hexasphere.calculate_hexagon_stats();
//! println!("Average hexagon radius: {:.3}", stats.average_hexagon_radius);
//! println!("Size variation: {:.1}%", 
//!     100.0 * (stats.max_hexagon_radius - stats.min_hexagon_radius) / stats.average_hexagon_radius);
//!
//! // Get regular hexagon approximations
//! let uniform_radius = hexasphere.get_uniform_hexagon_radius();
//! let orientations = hexasphere.get_hexagon_orientations();
//!
//! // Export for 3D visualization
//! let obj_content = hexasphere.to_obj();
//! std::fs::write("hexasphere.obj", obj_content).unwrap();
//! ```

pub mod geometry;
pub mod tile;
pub mod hexasphere;
pub mod approximation;
pub mod utils;

// Re-export main types for convenience
pub use hexasphere::{Hexasphere, HexagonStats};
pub use tile::{Tile, ThickTile};
pub use geometry::{Point, Vector3, Face};
pub use approximation::RegularHexagonParams;
pub use utils::{LatLon};
