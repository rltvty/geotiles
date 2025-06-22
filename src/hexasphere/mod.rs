//! Main hexasphere structure and construction algorithms.

pub mod core;
pub mod export;
pub mod hexagon_analyzer;
pub mod simplified;
pub mod statistics;
pub mod symmetrical;
pub mod true_symmetrical;

pub use core::Hexasphere;
pub use hexagon_analyzer::ShapeAnalyzer;
pub use simplified::{ShapeInstanceData, TileInstance, TileShape};
pub use statistics::HexagonStats;
pub use symmetrical::{create_symmetrical_hexasphere, SymmetricalConfig};
pub use true_symmetrical::{create_true_symmetrical_hexasphere, TrueSymmetricalConfig};
