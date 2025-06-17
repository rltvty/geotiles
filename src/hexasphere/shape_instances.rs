use crate::geometry::Point;
use crate::tile::{Tile, TileOrientation};
use crate::hexasphere::{Hexasphere, ShapeAnalyzer};

/// A unique tile shape with normalized vertices relative to origin
#[derive(Debug, Clone)]
pub struct TileShape {
    /// Vertices in local coordinates (relative to origin)
    pub vertices: Vec<Point>,
    /// Number of sides (5 for pentagon, 6 for hexagon)
    pub sides: usize,
    /// Average radius from center to vertices
    pub radius: f64,
}

/// An instance of a tile shape at a specific location with orientation
#[derive(Debug, Clone)]
pub struct TileInstance {
    /// Index of the shape template to use
    pub shape_index: usize,
    /// Center position of the tile
    pub center: Point,
    /// Orientation for placing the shape
    pub orientation: TileOrientation,
    /// Original tile index in the hexasphere
    pub tile_index: usize,
}

/// Collection of unique shapes and their instances
#[derive(Debug)]
pub struct ShapeInstanceData {
    /// Unique tile shapes
    pub shapes: Vec<TileShape>,
    /// All tile instances referencing the shapes
    pub instances: Vec<TileInstance>,
}

impl TileShape {
    /// Create a normalized shape from a tile
    fn from_tile(tile: &Tile) -> Self {
        let center = &tile.center_point;
        
        // Convert to local coordinates relative to center
        let vertices: Vec<Point> = tile.boundary
            .iter()
            .map(|p| Point::new(
                p.x - center.x,
                p.y - center.y,
                p.z - center.z
            ))
            .collect();
        
        // Calculate average radius
        let radius = vertices
            .iter()
            .map(|v| (v.x * v.x + v.y * v.y + v.z * v.z).sqrt())
            .sum::<f64>() / vertices.len() as f64;
        
        TileShape {
            vertices,
            sides: tile.boundary.len(),
            radius,
        }
    }
}

impl Hexasphere {
    /// Get unique tile shapes and their instances for efficient rendering.
    ///
    /// This method analyzes all tiles in the hexasphere and identifies unique shapes,
    /// returning a collection of shape templates and instances with proper orientations.
    /// This is ideal for instanced rendering where you want to create a few meshes
    /// and render them many times with different transformations.
    ///
    /// # Returns
    ///
    /// A `ShapeInstanceData` structure containing:
    /// - `shapes`: Vector of unique tile shapes with normalized vertices
    /// - `instances`: Vector of tile instances with shape indices and orientations
    ///
    /// # Shape Deduplication
    ///
    /// The algorithm groups tiles by their geometric properties:
    /// - Edge length patterns
    /// - Interior angles
    /// - Number of sides (pentagon vs hexagon)
    ///
    /// # Benefits
    ///
    /// - **Memory efficiency**: Store only unique shapes instead of all tiles
    /// - **Rendering performance**: Use GPU instancing for repeated shapes
    /// - **Simplified mesh creation**: Create meshes only for unique shapes
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// let hexasphere = Hexasphere::new(1.0, 4, 0.95);
    /// let shape_data = hexasphere.get_shape_instances();
    /// 
    /// println!("Unique shapes: {}", shape_data.shapes.len());
    /// println!("Total instances: {}", shape_data.instances.len());
    /// println!("Compression ratio: {:.1}x", 
    ///     shape_data.instances.len() as f64 / shape_data.shapes.len() as f64);
    /// 
    /// // Create meshes for each unique shape
    /// for (i, shape) in shape_data.shapes.iter().enumerate() {
    ///     println!("Shape {}: {} sides, radius {:.3}", i, shape.sides, shape.radius);
    ///     // create_mesh_from_shape(shape);
    /// }
    /// 
    /// // Place instances
    /// for instance in &shape_data.instances {
    ///     let transform = instance.orientation.to_transform_matrix(&instance.center);
    ///     // render_shape(shape_data.shapes[instance.shape_index], transform);
    /// }
    /// ```
    pub fn get_shape_instances(&self) -> ShapeInstanceData {
        let tiles = &self.tiles;
        let centers: Vec<_> = tiles.iter().map(|t| t.center_point.clone()).collect();
        
        // Analyze shapes
        let mut analyzer = ShapeAnalyzer::new();
        analyzer.analyze_tiles(tiles, &centers);
        
        let shape_indices = analyzer.get_shape_indices();
        let representatives = analyzer.get_shape_representatives();
        
        // Create unique shapes from representatives
        let shapes: Vec<TileShape> = representatives
            .iter()
            .map(|&tile_idx| TileShape::from_tile(&tiles[tile_idx]))
            .collect();
        
        // Create instances with orientations
        let instances: Vec<TileInstance> = tiles
            .iter()
            .enumerate()
            .filter_map(|(tile_idx, tile)| {
                tile.get_orientation().map(|orientation| {
                    TileInstance {
                        shape_index: shape_indices[tile_idx],
                        center: tile.center_point.clone(),
                        orientation,
                        tile_index: tile_idx,
                    }
                })
            })
            .collect();
        
        ShapeInstanceData { shapes, instances }
    }
    
    /// Get shape instances only for hexagonal tiles.
    ///
    /// Similar to `get_shape_instances` but filters out pentagons, returning only
    /// hexagonal shapes and their instances. This is useful when pentagons are
    /// handled separately or when you only need hexagonal tiles.
    ///
    /// # Returns
    ///
    /// A `ShapeInstanceData` structure containing only hexagonal shapes and instances
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// let hexasphere = Hexasphere::new(1.0, 3, 0.95);
    /// let hex_shapes = hexasphere.get_hexagon_shape_instances();
    /// 
    /// // All shapes will have 6 sides
    /// for shape in &hex_shapes.shapes {
    ///     assert_eq!(shape.sides, 6);
    /// }
    /// ```
    pub fn get_hexagon_shape_instances(&self) -> ShapeInstanceData {
        // Get all shape instances first
        let all_data = self.get_shape_instances();
        
        // Filter to only hexagonal shapes
        let hex_instances: Vec<_> = all_data.instances
            .into_iter()
            .filter(|instance| self.tiles[instance.tile_index].is_hexagon())
            .collect();
        
        // Find which shapes are actually used
        let mut used_shapes = std::collections::HashSet::new();
        for instance in &hex_instances {
            used_shapes.insert(instance.shape_index);
        }
        
        // Create mapping from old to new indices
        let mut shape_remap = std::collections::HashMap::new();
        let mut new_shapes = Vec::new();
        
        for (new_idx, &old_idx) in used_shapes.iter().enumerate() {
            shape_remap.insert(old_idx, new_idx);
            new_shapes.push(all_data.shapes[old_idx].clone());
        }
        
        // Update instances with new shape indices
        let remapped_instances: Vec<_> = hex_instances
            .into_iter()
            .map(|mut instance| {
                instance.shape_index = *shape_remap.get(&instance.shape_index).unwrap();
                instance
            })
            .collect();
        
        ShapeInstanceData {
            shapes: new_shapes,
            instances: remapped_instances,
        }
    }
    
    /// Get a uniform shape and all instance orientations for simplified rendering.
    ///
    /// Returns a single "average" hexagon shape and orientations for all tiles.
    /// This is the simplest approach when exact shape matching isn't critical.
    ///
    /// # Returns
    ///
    /// A tuple of `(TileShape, Vec<TileInstance>)` where all instances reference
    /// the same shape (shape_index = 0)
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// let hexasphere = Hexasphere::new(1.0, 3, 0.95);
    /// let (uniform_shape, instances) = hexasphere.get_uniform_shape_instances();
    /// 
    /// println!("Using single shape with radius {:.3}", uniform_shape.radius);
    /// println!("Placing {} instances", instances.len());
    /// ```
    pub fn get_uniform_shape_instances(&self) -> (TileShape, Vec<TileInstance>) {
        let stats = self.calculate_hexagon_stats();
        
        // Create an ideal hexagon shape
        let angle_step = std::f64::consts::PI * 2.0 / 6.0;
        let vertices: Vec<Point> = (0..6)
            .map(|i| {
                let angle = i as f64 * angle_step;
                Point::new(
                    stats.average_hexagon_radius * angle.cos(),
                    0.0,
                    stats.average_hexagon_radius * angle.sin()
                )
            })
            .collect();
        
        let uniform_shape = TileShape {
            vertices,
            sides: 6,
            radius: stats.average_hexagon_radius,
        };
        
        // Create instances for all valid tiles
        let instances: Vec<TileInstance> = self.tiles
            .iter()
            .enumerate()
            .filter_map(|(tile_idx, tile)| {
                tile.get_orientation().map(|orientation| {
                    TileInstance {
                        shape_index: 0, // All use the same shape
                        center: tile.center_point.clone(),
                        orientation,
                        tile_index: tile_idx,
                    }
                })
            })
            .collect();
        
        (uniform_shape, instances)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shape_instances() {
        let hs = Hexasphere::new(1.0, 3, 1.0);
        let shape_data = hs.get_shape_instances();
        
        // Should have fewer shapes than instances
        assert!(shape_data.shapes.len() < shape_data.instances.len());
        
        // All instances should reference valid shapes
        for instance in &shape_data.instances {
            assert!(instance.shape_index < shape_data.shapes.len());
        }
        
        // Check shape properties
        for shape in &shape_data.shapes {
            assert!(shape.sides == 5 || shape.sides == 6);
            assert!(shape.radius > 0.0);
            assert_eq!(shape.vertices.len(), shape.sides);
        }
    }
    
    #[test]
    fn test_hexagon_only_shapes() {
        let hs = Hexasphere::new(1.0, 3, 1.0);
        let hex_data = hs.get_hexagon_shape_instances();
        
        // All shapes should be hexagons
        for shape in &hex_data.shapes {
            assert_eq!(shape.sides, 6);
        }
        
        // All instances should be hexagons
        for instance in &hex_data.instances {
            assert!(hs.tiles[instance.tile_index].is_hexagon());
        }
    }
    
    #[test]
    fn test_uniform_shape() {
        let hs = Hexasphere::new(1.0, 3, 1.0);
        let (shape, instances) = hs.get_uniform_shape_instances();
        
        // Shape should be a regular hexagon
        assert_eq!(shape.sides, 6);
        assert_eq!(shape.vertices.len(), 6);
        
        // All instances should use shape index 0
        for instance in &instances {
            assert_eq!(instance.shape_index, 0);
        }
    }
}