//! Export functionality for hexasphere data.

use crate::hexasphere::core::Hexasphere;
use std::collections::HashMap;

impl Hexasphere {
    /// Exports the hexasphere as a JSON string.
    ///
    /// Provides a simple JSON representation of the hexasphere structure.
    /// For full JSON serialization with all tile data, consider using serde
    /// with appropriate derive macros.
    ///
    /// # Returns
    ///
    /// A JSON string containing basic hexasphere information
    ///
    /// # Current Format
    ///
    /// ```json
    /// {
    ///   "radius": 10.0,
    ///   "tile_count": 2562
    /// }
    /// ```
    ///
    /// # Future Enhancement
    ///
    /// For production use, consider implementing full serde serialization:
    /// ```rust,ignore
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Hexasphere { /* fields */ }
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// let json = hexasphere.to_json();
    /// println!("Hexasphere info: {}", json);
    ///
    /// // Save to file
    /// # fn save_example() -> std::io::Result<()> {
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// # let json = hexasphere.to_json();
    /// std::fs::write("hexasphere.json", json)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_json(&self) -> String {
        // This would require serde for proper JSON serialization
        // For now, return a simple string representation
        format!(
            "{{\"radius\": {}, \"tile_count\": {}}}",
            self.radius,
            self.tiles.len()
        )
    }

    /// Exports the hexasphere as a Wavefront OBJ file format string.
    ///
    /// Creates a complete 3D mesh file that can be loaded into 3D modeling software,
    /// game engines, or visualization tools. Each tile becomes a polygon face in the mesh.
    ///
    /// # Returns
    ///
    /// A string containing the complete OBJ file content
    ///
    /// # OBJ Format Structure
    ///
    /// ```obj
    /// # vertices
    /// v 1.234 5.678 9.012
    /// v 2.345 6.789 0.123
    /// ...
    ///
    /// # faces  
    /// f 1 2 3 4 5 6
    /// f 7 8 9 10 11
    /// ...
    /// ```
    ///
    /// # Features
    ///
    /// - **Vertex deduplication**: Shared vertices are reused (efficient)
    /// - **Polygon faces**: Each tile becomes one face (not triangulated)
    /// - **1-based indexing**: Follows OBJ standard (vertices start at 1)
    /// - **Mixed polygons**: Hexagons (6 vertices) and pentagons (5 vertices)
    ///
    /// # Compatible Software
    ///
    /// The generated OBJ files work with:
    /// - **3D Software**: Blender, Maya, 3ds Max, Cinema 4D
    /// - **Game Engines**: Unity, Unreal Engine, Godot
    /// - **CAD Software**: Fusion 360, SolidWorks (import)
    /// - **Web Libraries**: Three.js, Babylon.js
    /// - **Programming Libraries**: Open3D, MeshLab, etc.
    ///
    /// # Use Cases
    ///
    /// - Visual verification of hexasphere generation
    /// - 3D printing preparation
    /// - Game asset creation  
    /// - Scientific visualization
    /// - Educational demonstrations
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use geotiles::Hexasphere;
    /// # fn save_obj_example() -> std::io::Result<()> {
    /// # let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    /// let obj_content = hexasphere.to_obj();
    /// std::fs::write("geodesic_sphere.obj", obj_content)?;
    ///
    /// // Load in Blender: File -> Import -> Wavefront (.obj)
    /// // Load in Unity: Drag file into Assets folder
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Performance Notes
    ///
    /// - Generation time: O(n) where n is number of tiles
    /// - Memory usage: Temporary during generation, then just the string
    /// - File size: ~100 bytes per tile (varies with precision)
    pub fn to_obj(&self) -> String {
        let mut obj_text = String::from("# vertices\n");
        let mut vertices = Vec::new();
        let mut vertex_map = HashMap::new();
        let mut faces = Vec::new();

        for tile in &self.tiles {
            let mut face_indices = Vec::new();

            for boundary_point in &tile.boundary {
                let key = boundary_point.to_string();
                let index = if let Some(&existing_index) = vertex_map.get(&key) {
                    existing_index
                } else {
                    let new_index = vertices.len() + 1; // OBJ uses 1-based indexing
                    vertices.push(boundary_point.clone());
                    vertex_map.insert(key, new_index);
                    new_index
                };
                face_indices.push(index);
            }

            faces.push(face_indices);
        }

        // Write vertices
        for vertex in &vertices {
            obj_text.push_str(&format!("v {} {} {}\n", vertex.x, vertex.y, vertex.z));
        }

        // Write faces
        obj_text.push_str("\n# faces\n");
        for face in &faces {
            obj_text.push('f');
            for &index in face {
                obj_text.push_str(&format!(" {}", index));
            }
            obj_text.push('\n');
        }

        obj_text
    }
}

#[cfg(test)]
mod tests {
    use crate::hexasphere::core::Hexasphere;

    #[test]
    fn test_to_json_basic() {
        let hexasphere = Hexasphere::new(1.0, 1, 1.0);
        let json = hexasphere.to_json();

        assert!(json.contains("\"radius\": 1"));
        assert!(json.contains("\"tile_count\":"));
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
    }

    #[test]
    fn test_to_json_different_params() {
        let hexasphere = Hexasphere::new(5.0, 2, 0.8);
        let json = hexasphere.to_json();

        assert!(json.contains("\"radius\": 5"));
        assert!(json.contains("\"tile_count\":"));

        // Should have more tiles for higher subdivision
        let tile_count_str = json
            .split("\"tile_count\": ")
            .nth(1)
            .unwrap()
            .split('}')
            .next()
            .unwrap();
        let tile_count: usize = tile_count_str.parse().unwrap();
        assert!(tile_count > 10); // Should have a reasonable number of tiles
    }

    #[test]
    fn test_to_obj_structure() {
        let hexasphere = Hexasphere::new(1.0, 1, 1.0);
        let obj = hexasphere.to_obj();

        // Check basic OBJ format
        assert!(obj.contains("# vertices"));
        assert!(obj.contains("# faces"));
        assert!(obj.contains("v "));
        assert!(obj.contains("f "));

        // Check that vertices come before faces
        let vertex_pos = obj.find("# vertices").unwrap();
        let face_pos = obj.find("# faces").unwrap();
        assert!(vertex_pos < face_pos);
    }

    #[test]
    fn test_to_obj_vertex_format() {
        let hexasphere = Hexasphere::new(2.0, 1, 1.0);
        let obj = hexasphere.to_obj();

        // Find first vertex line
        let lines: Vec<&str> = obj.lines().collect();
        let vertex_line = lines
            .iter()
            .find(|line| line.starts_with("v "))
            .expect("Should have at least one vertex");

        // Should have exactly 4 parts: "v", x, y, z
        let parts: Vec<&str> = vertex_line.split_whitespace().collect();
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], "v");

        // Should be parseable as floats
        parts[1]
            .parse::<f64>()
            .expect("x coordinate should be float");
        parts[2]
            .parse::<f64>()
            .expect("y coordinate should be float");
        parts[3]
            .parse::<f64>()
            .expect("z coordinate should be float");
    }

    #[test]
    fn test_to_obj_face_format() {
        let hexasphere = Hexasphere::new(1.0, 1, 1.0);
        let obj = hexasphere.to_obj();

        // Find first face line
        let lines: Vec<&str> = obj.lines().collect();
        let face_line = lines
            .iter()
            .find(|line| line.starts_with("f "))
            .expect("Should have at least one face");

        // Should start with "f" and have multiple indices
        let parts: Vec<&str> = face_line.split_whitespace().collect();
        assert_eq!(parts[0], "f");
        assert!(parts.len() >= 4); // At least 3 vertices for a triangle

        // All indices should be positive integers (1-based)
        for i in 1..parts.len() {
            let index: usize = parts[i].parse().expect("Face index should be integer");
            assert!(index >= 1); // OBJ uses 1-based indexing
        }
    }

    #[test]
    fn test_to_obj_vertex_deduplication() {
        let hexasphere = Hexasphere::new(1.0, 1, 1.0);
        let obj = hexasphere.to_obj();

        // Count vertices and faces
        let vertex_count = obj.lines().filter(|line| line.starts_with("v ")).count();
        let face_count = obj.lines().filter(|line| line.starts_with("f ")).count();

        // Should have fewer vertices than total boundary points due to deduplication
        assert!(vertex_count > 0);
        assert!(face_count > 0);
        assert_eq!(face_count, hexasphere.tiles.len());

        // Verify vertex count is reasonable (icosahedron level 1 has specific structure)
        assert!(vertex_count < face_count * 6); // Should be deduplicated
    }

    #[test]
    fn test_to_obj_pentagon_hexagon_mix() {
        let hexasphere = Hexasphere::new(1.0, 2, 1.0);
        let obj = hexasphere.to_obj();

        // Count face sizes
        let mut pentagon_count = 0;
        let mut hexagon_count = 0;

        for line in obj.lines() {
            if line.starts_with("f ") {
                let vertex_count = line.split_whitespace().count() - 1; // Subtract "f"
                match vertex_count {
                    5 => pentagon_count += 1,
                    6 => hexagon_count += 1,
                    _ => panic!("Unexpected polygon with {} vertices", vertex_count),
                }
            }
        }

        // Should have exactly 12 pentagons and rest hexagons
        assert_eq!(pentagon_count, 12);
        assert!(hexagon_count > 0);
        assert_eq!(pentagon_count + hexagon_count, hexasphere.tiles.len());
    }
}
