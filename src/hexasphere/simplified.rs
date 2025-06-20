use crate::geometry::Point;
use crate::hexasphere::{Hexasphere, ShapeAnalyzer};
use crate::tile::{Tile, TileOrientation};

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
    /// Create a shape from a tile with vertices normalized to lie in the XY-plane.
    ///
    /// This method creates a flattened version of the tile shape where all vertices
    /// have Z=0, making it suitable for use with 2D shape systems like Bevy's
    /// ConvexPolygon. The tile's 3D orientation is "baked out" so that the
    /// TileInstance transform handles both positioning and rotation.
    ///
    /// # Arguments
    ///
    /// * `tile` - The tile to extract shape from
    /// * `orientation` - The tile's orientation for normalization
    ///
    /// # Returns
    ///
    /// A TileShape with vertices in the XY-plane (Z=0)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::{Hexasphere, TileShape};
    /// let hexasphere = Hexasphere::new(1.0, 2, 1.0);
    /// let tile = &hexasphere.tiles[0];
    /// let orientation = tile.get_orientation().unwrap();
    /// let normalized_shape = TileShape::from_tile_normalized(tile, &orientation);
    ///
    /// // All vertices will have Z ≈ 0
    /// for vertex in &normalized_shape.vertices {
    ///     assert!((vertex.z).abs() < 1e-10);
    /// }
    /// ```
    pub fn from_tile_normalized(tile: &Tile, orientation: &crate::tile::TileOrientation) -> Self {
        let center = &tile.center_point;

        // Convert to local coordinates relative to center
        let local_vertices: Vec<Point> = tile
            .boundary
            .iter()
            .map(|p| Point::new(p.x - center.x, p.y - center.y, p.z - center.z))
            .collect();

        // Create inverse transform to normalize to XY-plane
        // The orientation gives us the tile's local coordinate system
        let right = &orientation.right;
        let _up = &orientation.up;
        let forward = &orientation.forward;

        // Transform vertices to the tile's local 2D coordinate system
        let normalized_vertices: Vec<Point> = local_vertices
            .iter()
            .map(|local_vertex| {
                // Project onto the tile's right and forward axes (ignore up/normal)
                let local_x =
                    local_vertex.x * right.x + local_vertex.y * right.y + local_vertex.z * right.z;
                let local_y = local_vertex.x * forward.x
                    + local_vertex.y * forward.y
                    + local_vertex.z * forward.z;

                Point::new(local_x, local_y, 0.0) // Flattened to XY-plane
            })
            .collect();

        // Calculate average radius in 2D
        let radius = normalized_vertices
            .iter()
            .map(|v| (v.x * v.x + v.y * v.y).sqrt())
            .sum::<f64>()
            / normalized_vertices.len() as f64;

        TileShape {
            vertices: normalized_vertices,
            sides: tile.boundary.len(),
            radius,
        }
    }

    /// Create a normalized shape from a tile
    fn from_tile(tile: &Tile) -> Self {
        let center = &tile.center_point;

        // Convert to local coordinates relative to center
        let vertices: Vec<Point> = tile
            .boundary
            .iter()
            .map(|p| Point::new(p.x - center.x, p.y - center.y, p.z - center.z))
            .collect();

        // Calculate average radius
        let radius = vertices
            .iter()
            .map(|v| (v.x * v.x + v.y * v.y + v.z * v.z).sqrt())
            .sum::<f64>()
            / vertices.len() as f64;

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
                tile.get_orientation().map(|orientation| TileInstance {
                    shape_index: shape_indices[tile_idx],
                    center: tile.center_point.clone(),
                    orientation,
                    tile_index: tile_idx,
                })
            })
            .collect();

        ShapeInstanceData { shapes, instances }
    }

    /// Get normalized shape instances with vertices flattened to the XY-plane.
    ///
    /// This method creates shape instances where each TileShape has vertices
    /// normalized to lie in the XY-plane (Z=0), making them suitable for use
    /// with 2D shape systems like Bevy's ConvexPolygon. The original 3D
    /// orientation is "baked out" of the shape geometry.
    ///
    /// # Arguments
    ///
    /// * `max_shapes` - Maximum number of unique hexagon shapes to create (pentagons are always separate)
    /// * `tolerance` - Geometric error tolerance for shape clustering (0.0 = exact, 1.0 = very loose)
    ///
    /// # Returns
    ///
    /// A `ShapeInstanceData` structure with normalized shapes
    ///
    /// # Benefits for Bevy Integration
    ///
    /// - **Compatible with ConvexPolygon**: Shapes can be used directly with Bevy's 2D shape system
    /// - **Simplified rendering**: No need for custom mesh generation
    /// - **Potential performance gains**: Leverage Bevy's optimized shape rendering
    /// - **Controllable complexity**: Limit number of unique shapes for performance
    ///
    /// # Example
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// let hexasphere = Hexasphere::new(1.0, 4, 0.95);
    /// let shape_data = hexasphere.get_normalized_shape_instances(15, 0.05);
    ///
    /// // All shape vertices will have Z ≈ 0
    /// for shape in &shape_data.shapes {
    ///     for vertex in &shape.vertices {
    ///         assert!((vertex.z).abs() < 1e-10);
    ///     }
    /// }
    ///
    /// // Use with Bevy ConvexPolygon:
    /// // let points: Vec<Vec2> = shape.vertices.iter()
    /// //     .map(|v| Vec2::new(v.x as f32, v.y as f32))
    /// //     .collect();
    /// // let polygon = ConvexPolygon::new(points);
    /// ```
    pub fn get_normalized_shape_instances(
        &self,
        max_shapes: usize,
        tolerance: f64,
    ) -> ShapeInstanceData {
        // Get natural shape instances first
        let natural_shapes = self.get_shape_instances();
        
        // Separate pentagons and hexagons
        let (pentagon_shapes, hexagon_shapes): (Vec<_>, Vec<_>) = natural_shapes
            .shapes
            .iter()
            .enumerate()
            .partition(|(_, shape)| shape.sides == 5);
        
        // Count hexagon shapes for comparison
        let hexagon_shape_count = hexagon_shapes.len();
        
        if max_shapes >= hexagon_shape_count {
            // No simplification needed - just normalize all shapes
            let normalized_shapes: Vec<TileShape> = natural_shapes.shapes
                .iter()
                .enumerate()
                .map(|(shape_idx, _)| {
                    // Find a representative instance for this shape to get the tile
                    if let Some(instance) = natural_shapes.instances
                        .iter()
                        .find(|inst| inst.shape_index == shape_idx)
                    {
                        let tile = &self.tiles[instance.tile_index];
                        if let Some(orientation) = tile.get_orientation() {
                            TileShape::from_tile_normalized(tile, &orientation)
                        } else {
                            natural_shapes.shapes[shape_idx].clone()
                        }
                    } else {
                        natural_shapes.shapes[shape_idx].clone()
                    }
                })
                .collect();
            
            return ShapeInstanceData {
                shapes: normalized_shapes,
                instances: natural_shapes.instances,
            };
        }
        
        // Cluster hexagon shapes by radius similarity
        let hexagon_clusters = self.cluster_hexagons_by_radius(&hexagon_shapes, max_shapes, tolerance);
        
        // Build simplified shapes and instances
        let mut simplified_shapes = Vec::new();
        let mut simplified_instances = Vec::new();
        
        // Add all pentagon shapes first (never simplified)
        for (original_idx, _pentagon_shape) in pentagon_shapes {
            // Create normalized pentagon shape
            if let Some(instance) = natural_shapes.instances
                .iter()
                .find(|inst| inst.shape_index == original_idx)
            {
                let tile = &self.tiles[instance.tile_index];
                if let Some(orientation) = tile.get_orientation() {
                    simplified_shapes.push(TileShape::from_tile_normalized(tile, &orientation));
                } else {
                    simplified_shapes.push(natural_shapes.shapes[original_idx].clone());
                }
            } else {
                simplified_shapes.push(natural_shapes.shapes[original_idx].clone());
            }
            
            let shape_index = simplified_shapes.len() - 1;
            
            // Find all instances that used this pentagon shape
            for instance in &natural_shapes.instances {
                if instance.shape_index == original_idx {
                    let mut new_instance = instance.clone();
                    new_instance.shape_index = shape_index;
                    simplified_instances.push(new_instance);
                }
            }
        }
        
        // Add clustered hexagon shapes
        for cluster in hexagon_clusters {
            // Create normalized representative shape
            if let Some(instance) = natural_shapes.instances
                .iter()
                .find(|inst| inst.shape_index == cluster.members[0].original_index)
            {
                let tile = &self.tiles[instance.tile_index];
                if let Some(orientation) = tile.get_orientation() {
                    simplified_shapes.push(TileShape::from_tile_normalized(tile, &orientation));
                } else {
                    simplified_shapes.push(cluster.representative.clone());
                }
            } else {
                simplified_shapes.push(cluster.representative.clone());
            }
            
            let shape_index = simplified_shapes.len() - 1;
            
            // Add instances for all shapes in this cluster
            for member in &cluster.members {
                for instance in &natural_shapes.instances {
                    if instance.shape_index == member.original_index {
                        let mut new_instance = instance.clone();
                        new_instance.shape_index = shape_index;
                        simplified_instances.push(new_instance);
                    }
                }
            }
        }
        
        ShapeInstanceData {
            shapes: simplified_shapes,
            instances: simplified_instances,
        }
    }


    /// Helper method to cluster hexagon shapes by radius similarity
    fn cluster_hexagons_by_radius(
        &self,
        hexagon_shapes: &[(usize, &TileShape)],
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
            });
        }

        clusters
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
        let hex_instances: Vec<_> = all_data
            .instances
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
                    stats.average_hexagon_radius * angle.sin(),
                )
            })
            .collect();

        let uniform_shape = TileShape {
            vertices,
            sides: 6,
            radius: stats.average_hexagon_radius,
        };

        // Create instances for all valid tiles
        let instances: Vec<TileInstance> = self
            .tiles
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


/// A cluster of similar hexagon shapes with a representative
#[derive(Debug, Clone)]
struct HexagonCluster {
    /// The representative shape for this cluster
    representative: TileShape,
    /// All shapes that belong to this cluster
    members: Vec<ClusterMember>,
}

/// A member of a hexagon cluster
#[derive(Debug, Clone)]
struct ClusterMember {
    /// Original index of this shape in the natural shape array
    original_index: usize,
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

    #[test]
    fn test_normalized_shapes() {
        let hs = Hexasphere::new(1.0, 2, 1.0);
        let max_shapes = 10;
        let tolerance = 0.05;
        let shape_data = hs.get_normalized_shape_instances(max_shapes, tolerance);

        // Should have shapes and instances
        assert!(!shape_data.shapes.is_empty());
        assert!(!shape_data.instances.is_empty());

        // All shapes should be flattened to XY-plane (Z ≈ 0)
        for shape in &shape_data.shapes {
            for vertex in &shape.vertices {
                assert!(
                    vertex.z.abs() < 1e-10,
                    "Vertex Z={} should be near zero for normalized shape",
                    vertex.z
                );
            }

            // Should still have proper geometry in XY
            assert!(shape.vertices.len() >= 5); // At least pentagon
            assert!(shape.radius > 0.0);
        }

        // Verify instance count matches tile count
        assert_eq!(shape_data.instances.len(), hs.tiles.len());
    }

    #[test]
    fn test_normalized_tile_shape() {
        let hs = Hexasphere::new(1.0, 2, 1.0);
        let tile = &hs.tiles[0];

        if let Some(orientation) = tile.get_orientation() {
            let normalized_shape = TileShape::from_tile_normalized(tile, &orientation);

            // All vertices should have Z ≈ 0
            for vertex in &normalized_shape.vertices {
                assert!(
                    vertex.z.abs() < 1e-10,
                    "Normalized vertex Z={} should be near zero",
                    vertex.z
                );
            }

            // Should preserve basic properties
            assert_eq!(normalized_shape.sides, tile.boundary.len());
            assert!(normalized_shape.radius > 0.0);
            assert_eq!(normalized_shape.vertices.len(), tile.boundary.len());
        }
    }
}
