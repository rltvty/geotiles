use geotiles::hexasphere::Hexasphere;
use std::fs::File;
use std::io::Write;

fn main() {
    println!("Generating instanced hexasphere data...\n");
    
    // Generate hexasphere at different subdivision levels
    for subdivisions in 2..=5 {
        let hs = Hexasphere::new(1.0, subdivisions, 1.0);
        let instanced = hs.export_instanced(subdivisions as u32);
        
        println!("Subdivision level {}: ", subdivisions);
        println!("  Total tiles: {}", instanced.instances.len());
        println!("  Unique shapes: {}", instanced.shapes.len());
        println!("  Compression ratio: {:.1}x", 
            instanced.instances.len() as f64 / instanced.shapes.len() as f64);
        
        // Show shape distribution
        println!("  Shape distribution:");
        for (i, shape) in instanced.shapes.iter().take(5).enumerate() {
            println!("    Shape {}: {} instances ({:.1}%)", 
                i, 
                shape.instance_count,
                100.0 * shape.instance_count as f64 / instanced.instances.len() as f64
            );
        }
        
        // Save to JSON file
        let filename = format!("instanced_hexasphere_s{}.json", subdivisions);
        let json = serde_json::to_string_pretty(&instanced).unwrap();
        let mut file = File::create(&filename).unwrap();
        file.write_all(json.as_bytes()).unwrap();
        println!("  Saved to: {}", filename);
        println!();
    }
    
    // Show example usage for Bevy
    println!("Example Bevy usage:");
    println!("```rust");
    println!("// Load the instanced data");
    println!("let instanced_data: InstancedHexasphere = serde_json::from_str(&json_string)?;");
    println!();
    println!("// Create meshes for each unique shape");
    println!("let mut shape_meshes = Vec::new();");
    println!("for shape in &instanced_data.shapes {{");
    println!("    let mesh = create_mesh_from_vertices(&shape.vertices);");
    println!("    shape_meshes.push(meshes.add(mesh));");
    println!("}}");
    println!();
    println!("// Spawn instances");
    println!("for instance in &instanced_data.instances {{");
    println!("    commands.spawn(PbrBundle {{");
    println!("        mesh: shape_meshes[instance.shape_id].clone(),");
    println!("        transform: Transform {{");
    println!("            translation: Vec3::from(instance.center),");
    println!("            rotation: Quat::from_xyzw(");
    println!("                instance.rotation[0],");
    println!("                instance.rotation[1],");
    println!("                instance.rotation[2],");
    println!("                instance.rotation[3],");
    println!("            ),");
    println!("            scale: Vec3::splat(instance.scale),");
    println!("        }},");
    println!("        ..default()");
    println!("    }});");
    println!("}}");
    println!("```");
}