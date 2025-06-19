//! Main hexasphere structure and construction algorithms.
//!
//! This module provides the core functionality for generating geodesic polyhedra
//! (Goldberg polyhedra) with hexagonal and pentagonal tiles.
//!
//! ## Features
//!
//! - **Basic generation**: Create hexaspheres with customizable subdivision levels
//! - **Shape analysis**: Analyze and measure tile uniformity
//! - **Shape instancing**: Identify unique shapes for efficient rendering
//! - **Export formats**: JSON and OBJ file output
//! - **Regular approximations**: Get uniform hexagon parameters
//!
//! ## Shape Instancing (NEW!)
//!
//! The shape instancing feature identifies that many tiles share the same shape,
//! just rotated and positioned differently. This enables significant optimizations:
//!
//! ```rust
//! use geotiles::Hexasphere;
//!
//! let hexasphere = Hexasphere::new(1.0, 4, 0.95);
//! let shape_data = hexasphere.get_shape_instances();
//!
//! // At subdivision 4: ~162 tiles reduced to ~15 unique shapes (10x reduction!)
//! println!("Unique shapes: {}", shape_data.shapes.len());
//! println!("Total instances: {}", shape_data.instances.len());
//! ```

pub mod core;
pub mod direct_shapes;
pub mod export;
pub mod high_subdivision;
pub mod shape_analysis;
pub mod shape_export;
pub mod shape_instances;
pub mod shape_theory;
pub mod statistics;

pub use core::{Hexasphere, HexagonInstance, SimplificationStats};
pub use shape_analysis::{HexagonShape, ShapeAnalyzer};
pub use shape_export::{InstancedHexasphere, ShapeInstance, ShapeTemplate};
pub use shape_instances::{ShapeInstanceData, TileInstance, TileShape};
pub use statistics::HexagonStats;
