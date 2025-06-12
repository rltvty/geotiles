//! Tile structures and operations.
//!
//! This module contains the tile representations and all operations
//! for working with polygonal tiles on the sphere surface.

pub mod orientation;
pub mod thick_tile;
pub mod tile;

pub use orientation::TileOrientation;
pub use thick_tile::{ThickTile, ThickTileVertices};
pub use tile::Tile;
