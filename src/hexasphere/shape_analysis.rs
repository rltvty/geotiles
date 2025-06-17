use crate::geometry::{Point, Vector3};
use crate::tile::Tile;
use std::collections::HashMap;

/// A normalized hexagon shape, centered at origin with a canonical orientation
#[derive(Debug, Clone)]
pub struct HexagonShape {
    /// Vertices relative to center, sorted by angle
    pub vertices: Vec<Vector3>,
    /// Edge lengths in order
    pub edge_lengths: Vec<f64>,
    /// Interior angles in order
    pub angles: Vec<f64>,
}

impl HexagonShape {
    /// Create a normalized shape from a tile
    pub fn from_tile(tile: &Tile, center: &Point) -> Self {
        // Translate vertices to be relative to center
        let relative_vertices: Vec<Vector3> = tile.boundary
            .iter()
            .map(|p| Vector3::new(p.x - center.x, p.y - center.y, p.z - center.z))
            .collect();
        
        // Calculate edge lengths
        let mut edge_lengths = Vec::new();
        for i in 0..relative_vertices.len() {
            let next = (i + 1) % relative_vertices.len();
            let edge_length = (relative_vertices[next] - relative_vertices[i]).magnitude();
            edge_lengths.push(edge_length);
        }
        
        // Calculate interior angles
        let mut angles = Vec::new();
        for i in 0..relative_vertices.len() {
            let prev = if i == 0 { relative_vertices.len() - 1 } else { i - 1 };
            let next = (i + 1) % relative_vertices.len();
            
            let v1 = relative_vertices[prev] - relative_vertices[i];
            let v2 = relative_vertices[next] - relative_vertices[i];
            
            let angle = v1.angle_to(&v2);
            angles.push(angle);
        }
        
        // Normalize the shape by rotating to a canonical orientation
        // We'll use the longest edge as the reference
        let (longest_idx, _) = edge_lengths
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();
        
        // Rotate vertices so longest edge starts at index 0
        let mut normalized_vertices = Vec::new();
        for i in 0..relative_vertices.len() {
            let idx = (i + longest_idx) % relative_vertices.len();
            normalized_vertices.push(relative_vertices[idx]);
        }
        
        // Also rotate edge lengths and angles
        let mut normalized_edges = Vec::new();
        let mut normalized_angles = Vec::new();
        for i in 0..edge_lengths.len() {
            let idx = (i + longest_idx) % edge_lengths.len();
            normalized_edges.push(edge_lengths[idx]);
            normalized_angles.push(angles[idx]);
        }
        
        HexagonShape {
            vertices: normalized_vertices,
            edge_lengths: normalized_edges,
            angles: normalized_angles,
        }
    }
    
    /// Check if two shapes are similar within a tolerance
    pub fn is_similar_to(&self, other: &HexagonShape, tolerance: f64) -> bool {
        if self.vertices.len() != other.vertices.len() {
            return false;
        }
        
        // Compare edge length patterns
        for i in 0..self.edge_lengths.len() {
            if (self.edge_lengths[i] - other.edge_lengths[i]).abs() > tolerance {
                return false;
            }
        }
        
        // Compare angle patterns
        for i in 0..self.angles.len() {
            if (self.angles[i] - other.angles[i]).abs() > tolerance {
                return false;
            }
        }
        
        true
    }
    
    /// Generate a hash key based on discretized shape properties
    pub fn shape_key(&self, precision: usize) -> String {
        let mut key = format!("v{}_", self.vertices.len());
        
        // Discretize edge lengths
        for edge in &self.edge_lengths {
            let discretized = (edge * 10f64.powi(precision as i32)).round() as i64;
            key.push_str(&format!("e{}_", discretized));
        }
        
        // Discretize angles
        for angle in &self.angles {
            let discretized = (angle.to_degrees() * 10f64.powi(precision as i32)).round() as i64;
            key.push_str(&format!("a{}_", discretized));
        }
        
        key
    }
}

/// Analyzes hexagon shapes in a hexasphere and groups them by similarity
pub struct ShapeAnalyzer {
    shapes: Vec<HexagonShape>,
    shape_groups: HashMap<String, Vec<usize>>,
}

impl Default for ShapeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ShapeAnalyzer {
    pub fn new() -> Self {
        ShapeAnalyzer {
            shapes: Vec::new(),
            shape_groups: HashMap::new(),
        }
    }
    
    /// Analyze all tiles in a hexasphere and group by shape
    pub fn analyze_tiles(&mut self, tiles: &[Tile], centers: &[Point]) {
        self.shapes.clear();
        self.shape_groups.clear();
        
        // Create shapes for all tiles
        for (i, tile) in tiles.iter().enumerate() {
            let shape = HexagonShape::from_tile(tile, &centers[i]);
            self.shapes.push(shape);
        }
        
        // Group similar shapes
        let precision = 3; // 3 decimal places for grouping
        for (i, shape) in self.shapes.iter().enumerate() {
            let key = shape.shape_key(precision);
            self.shape_groups.entry(key).or_default().push(i);
        }
    }
    
    /// Get unique shape count
    pub fn unique_shape_count(&self) -> usize {
        self.shape_groups.len()
    }
    
    /// Get shape distribution statistics
    pub fn get_shape_distribution(&self) -> Vec<(String, usize)> {
        let mut distribution: Vec<_> = self.shape_groups
            .iter()
            .map(|(key, indices)| (key.clone(), indices.len()))
            .collect();
        
        // Sort by count descending
        distribution.sort_by(|a, b| b.1.cmp(&a.1));
        
        distribution
    }
    
    /// Get the shape index for each tile
    pub fn get_shape_indices(&self) -> Vec<usize> {
        let mut shape_index_map = HashMap::new();
        
        // Assign indices to unique shapes
        for (next_index, key) in self.shape_groups.keys().enumerate() {
            shape_index_map.insert(key.clone(), next_index);
        }
        
        // Map each tile to its shape index
        let mut indices = vec![0; self.shapes.len()];
        for (i, shape) in self.shapes.iter().enumerate() {
            let key = shape.shape_key(3);
            indices[i] = *shape_index_map.get(&key).unwrap();
        }
        
        indices
    }
    
    /// Get representative tiles for each unique shape
    pub fn get_shape_representatives(&self) -> Vec<usize> {
        self.shape_groups
            .values()
            .map(|indices| indices[0])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hexasphere::Hexasphere;
    
    #[test]
    fn test_shape_analysis_subdivision_2() {
        let hs = Hexasphere::new(1.0, 2, 1.0);
        let tiles = &hs.tiles;
        let centers: Vec<_> = tiles.iter().map(|t| t.center_point.clone()).collect();
        
        let mut analyzer = ShapeAnalyzer::new();
        analyzer.analyze_tiles(tiles, &centers);
        
        let unique_count = analyzer.unique_shape_count();
        println!("Subdivision 2: {} unique shapes", unique_count);
        
        // For subdivision 2 (soccer ball), we expect around 5 unique shapes
        assert_eq!(unique_count, 5);
    }
    
    #[test]
    fn test_shape_distribution() {
        let hs = Hexasphere::new(1.0, 3, 1.0);
        let tiles = &hs.tiles;
        let centers: Vec<_> = tiles.iter().map(|t| t.center_point.clone()).collect();
        
        let mut analyzer = ShapeAnalyzer::new();
        analyzer.analyze_tiles(tiles, &centers);
        
        let distribution = analyzer.get_shape_distribution();
        println!("Shape distribution for subdivision 3:");
        for (key, count) in distribution.iter().take(5) {
            println!("  Shape {}: {} tiles", key, count);
        }
    }
}