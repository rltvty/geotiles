use geotiles::Hexasphere;

fn main() {
    println!("Demonstrating shape instance API for efficient rendering\n");
    
    // Test different subdivision levels
    for subdivisions in 2..=5 {
        println!("=== Subdivision level {} ===", subdivisions);
        let hexasphere = Hexasphere::new(1.0, subdivisions, 0.95);
        
        // Method 1: Get all unique shapes and instances
        let shape_data = hexasphere.get_shape_instances();
        println!("\n1. All shapes and instances:");
        println!("   Unique shapes: {}", shape_data.shapes.len());
        println!("   Total instances: {}", shape_data.instances.len());
        println!("   Compression ratio: {:.1}x", 
            shape_data.instances.len() as f64 / shape_data.shapes.len() as f64);
        
        // Show shape distribution
        let mut shape_counts = vec![0; shape_data.shapes.len()];
        for instance in &shape_data.instances {
            shape_counts[instance.shape_index] += 1;
        }
        
        println!("   Shape distribution:");
        for (i, count) in shape_counts.iter().enumerate().take(5) {
            let shape = &shape_data.shapes[i];
            println!("     Shape {}: {} sides, {} instances", i, shape.sides, count);
        }
        
        // Method 2: Get hexagon-only shapes
        let hex_data = hexasphere.get_hexagon_shape_instances();
        println!("\n2. Hexagon-only shapes:");
        println!("   Unique hexagon shapes: {}", hex_data.shapes.len());
        println!("   Hexagon instances: {}", hex_data.instances.len());
        
        // Method 3: Get uniform shape with all orientations
        let (uniform_shape, instances) = hexasphere.get_uniform_shape_instances();
        println!("\n3. Uniform shape approach:");
        println!("   Single shape radius: {:.4}", uniform_shape.radius);
        println!("   Total instances: {}", instances.len());
        
        // Show example usage for Bevy
        println!("\n4. Example Bevy usage:");
        println!("   ```rust");
        println!("   // Create meshes for unique shapes");
        println!("   let mut shape_meshes = vec![];");
        println!("   for shape in &shape_data.shapes {{");
        println!("       let mesh = create_tile_mesh(&shape.vertices);");
        println!("       shape_meshes.push(meshes.add(mesh));");
        println!("   }}");
        println!();
        println!("   // Spawn instances");
        println!("   for instance in &shape_data.instances {{");
        println!("       let transform = instance.orientation.to_transform_matrix(&instance.center);");
        println!("       commands.spawn(PbrBundle {{");
        println!("           mesh: shape_meshes[instance.shape_index].clone(),");
        println!("           transform: Transform::from_matrix(transform),");
        println!("           ..default()");
        println!("       }});");
        println!("   }}");
        println!("   ```");
        println!();
    }
    
    // Performance comparison
    println!("=== Performance Benefits ===");
    println!("\nMemory usage comparison (subdivision 5):");
    let hs = Hexasphere::new(1.0, 5, 0.95);
    let shape_data = hs.get_shape_instances();
    
    let vertices_per_tile = 6; // Average
    let floats_per_vertex = 3;
    let bytes_per_float = 4;
    
    let traditional_memory = hs.tiles.len() * vertices_per_tile * floats_per_vertex * bytes_per_float;
    let instanced_memory = shape_data.shapes.len() * vertices_per_tile * floats_per_vertex * bytes_per_float;
    
    println!("Traditional approach: {} tiles × {} vertices × {} floats × {} bytes = {} KB",
        hs.tiles.len(), vertices_per_tile, floats_per_vertex, bytes_per_float,
        traditional_memory / 1024);
    
    println!("Instanced approach: {} shapes × {} vertices × {} floats × {} bytes = {} KB",
        shape_data.shapes.len(), vertices_per_tile, floats_per_vertex, bytes_per_float,
        instanced_memory / 1024);
    
    println!("Memory savings: {:.1}% reduction", 
        100.0 * (1.0 - instanced_memory as f64 / traditional_memory as f64));
}