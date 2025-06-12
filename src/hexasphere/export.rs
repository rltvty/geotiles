//! Export functionality for hexasphere data.

use crate::hexasphere::Hexasphere;
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
    /// ```rust
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Hexasphere { ... }
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// let json = hexasphere.to_json();
    /// println!("Hexasphere info: {}", json);
    ///
    /// // Save to file
    /// std::fs::write("hexasphere.json", json)?;
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
    /// let obj_content = hexasphere.to_obj();
    /// std::fs::write("geodesic_sphere.obj", obj_content)?;
    ///
    /// // Load in Blender: File -> Import -> Wavefront (.obj)
    /// // Load in Unity: Drag file into Assets folder
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
