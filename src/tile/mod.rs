//! Tile structures and operations.
//!
//! This module contains the tile representations and all operations
//! for working with polygonal tiles on the sphere surface.

pub mod tile;
pub mod thick_tile;
pub mod orientation;

pub use tile::Tile;
pub use thick_tile::{ThickTile, ThickTileVertices};
pub use orientation::TileOrientation;
