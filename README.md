# Geotiles 🌐

A Rust library for generating geodesic polyhedra (Goldberg polyhedra) by subdividing an icosahedron and projecting it onto a sphere. Creates sphere-like surfaces composed of mostly hexagonal tiles with exactly 12 pentagonal tiles.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Documentation](https://docs.rs/geotiles/badge.svg)](https://docs.rs/geotiles)

## 🎯 Features

- **Geodesic Polyhedron Generation**: Create detailed sphere approximations using icosahedral subdivision
- **Regular Hexagon Approximation**: Generate uniform hexagons for consistent tile-based applications
- **Thick Tile Support**: Extrude tiles inward for 3D visualization and manufacturing
- **Statistical Analysis**: Comprehensive metrics for size variation and approximation quality
- **Multiple Export Formats**: JSON and OBJ file export for visualization and integration
- **Orientation Calculations**: Local coordinate systems for proper tile placement
- **Comprehensive Documentation**: Detailed API docs with mathematical background

## 🚀 Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
geotiles = "0.1.0"
```

### Basic Usage

```rust
use geotiles::Hexasphere;

// Create a hexasphere with radius 10, 4 subdivision levels, 90% tile size
let hexasphere = Hexasphere::new(10.0, 4, 0.9);

println!("Generated {} tiles", hexasphere.tiles.len());

// Export as OBJ file for 3D visualization
let obj_content = hexasphere.to_obj();
std::fs::write("geodesic_sphere.obj", obj_content)?;
```

### Regular Hexagon Approximation

```rust
// Analyze hexagon uniformity
let stats = hexasphere.calculate_hexagon_stats();
println!("Size variation: {:.1}%", 
    100.0 * stats.radius_std_deviation / stats.average_hexagon_radius);

// Get uniform size for all hexagons
let uniform_radius = hexasphere.get_uniform_hexagon_radius();

// Get orientations for proper placement
let orientations = hexasphere.get_hexagon_orientations();

for (tile, orientation) in hexasphere.tiles.iter().zip(orientations.iter()) {
    if tile.is_hexagon() {
        let transform = orientation.to_transform_matrix(&tile.center_point);
        // Use transform matrix in your 3D engine
    }
}
```

### Thick 3D Tiles

```rust
// Create tiles with 0.5 unit thickness
let thick_tiles = hexasphere.create_thick_tiles(0.5);

for thick_tile in thick_tiles {
    // Generate complete 3D mesh with vertices and indices
    let mesh_data = thick_tile.generate_all_vertices();
    
    // Use mesh_data.vertices and mesh_data.indices in your renderer
    create_3d_mesh(mesh_data.vertices, mesh_data.indices);
}
```

## 📊 Mathematical Background

### Geodesic Polyhedra

Geodesic polyhedra are created by:

1. **Starting with an icosahedron** (20 triangular faces, 12 vertices)
2. **Subdividing each triangle** into smaller triangles (4^n growth)
3. **Projecting vertices** onto a sphere surface
4. **Creating dual polyhedron** where vertices become tile centers
5. **Forming tile boundaries** using triangle face centroids

### Why 12 Pentagons?

Due to Euler's formula for polyhedra (V - E + F = 2), it's mathematically impossible to tile a sphere with only regular hexagons. Exactly 12 pentagons are required, positioned at the vertices of the original icosahedron.

### Subdivision Levels

| Level | Tiles | Faces | Performance |
|-------|-------|-------|-------------|
| 0     | 12    | 20    | Instant     |
| 1     | 42    | 80    | < 1ms       |
| 2     | 162   | 320   | < 1ms       |
| 3     | 642   | 1,280 | < 10ms      |
| 4     | 2,562 | 5,120 | < 100ms     |
| 5     | 10,242| 20,480| < 1s        |

## 🎮 Applications

### Game Development
- **Spherical game boards**: Turn-based strategy on sphere surfaces
- **Planet generation**: Procedural planet surfaces with tile-based regions
- **Space games**: Spherical coordinate systems for orbital mechanics

### Scientific Visualization
- **Global data mapping**: Climate data, population density, geological surveys
- **Astronomical applications**: Sphere tessellation for sky maps
- **Simulation grids**: Discrete sphere surfaces for numerical simulations

### Architecture & Manufacturing
- **Geodesic domes**: Structural design with optimized load distribution
- **3D printing**: Spherical objects with controlled polygon density
- **Architectural modeling**: Dome and sphere structures

### Computer Graphics
- **Level-of-detail**: Multiple resolution sphere meshes
- **Texture mapping**: UV unwrapping for spherical objects
- **Collision detection**: Simplified sphere approximations

## 🔧 Integration Examples

### Bevy Game Engine

```rust
use bevy::prelude::*;
use geotiles::Hexasphere;

fn setup_hexasphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let hexasphere = Hexasphere::new(5.0, 4, 0.9);
    let uniform_radius = hexasphere.get_uniform_hexagon_radius();
    
    for tile in &hexasphere.tiles {
        if tile.is_hexagon() {
            if let Some(orientation) = tile.get_orientation() {
                let transform = Transform {
                    translation: Vec3::new(
                        tile.center_point.x as f32,
                        tile.center_point.y as f32, 
                        tile.center_point.z as f32
                    ),
                    rotation: orientation_to_quat(&orientation),
                    scale: Vec3::ONE,
                };
                
                commands.spawn(PbrBundle {
                    mesh: meshes.add(create_hexagon_mesh(uniform_radius as f32)),
                    material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                    transform,
                    ..default()
                });
            }
        }
    }
}
```

### Three.js / WebGL

```javascript
// Convert Rust output to JavaScript
const hexasphereData = JSON.parse(rustHexasphereOutput);
const orientations = rustOrientations;

hexasphereData.tiles.forEach((tile, index) => {
    if (tile.boundary.length === 6) { // Hexagon
        const geometry = new THREE.CylinderGeometry(uniformRadius, uniformRadius, 0.1, 6);
        const material = new THREE.MeshLambertMaterial({ color: 0x00ff00 });
        const mesh = new THREE.Mesh(geometry, material);
        
        // Apply position and rotation from Rust calculations
        mesh.position.set(tile.center.x, tile.center.y, tile.center.z);
        mesh.rotation.setFromRotationMatrix(orientations[index]);
        
        scene.add(mesh);
    }
});
```

## 📈 Performance Guidelines

### Subdivision Level Selection

- **Interactive applications**: Levels 2-4 (162-2,562 tiles)
- **High-quality visualization**: Levels 4-6 (2,562-10,242 tiles)
- **Scientific simulation**: Levels 5-7 (10,242-40,962 tiles)
- **Real-time games**: Levels 1-3 (42-642 tiles)

### Memory Optimization

```rust
// Cache hexasphere for repeated use
static CACHED_HEXASPHERE: OnceCell<Hexasphere> = OnceCell::new();

fn get_hexasphere() -> &'static Hexasphere {
    CACHED_HEXASPHERE.get_or_init(|| {
        Hexasphere::new(10.0, 4, 0.9)
    })
}

// Use approximation for uniform applications
let uniform_radius = hexasphere.get_uniform_hexagon_radius();
let orientations = hexasphere.get_hexagon_orientations();
// Store only these smaller data structures instead of full tiles
```

## 🔬 Quality Assessment

### Hexagon Regularity Analysis

```rust
let stats = hexasphere.calculate_hexagon_stats();

// Check uniformity
let variation_percent = 100.0 * stats.radius_std_deviation / stats.average_hexagon_radius;
match variation_percent {
    v if v < 5.0  => println!("Excellent uniformity - regular hexagons work great"),
    v if v < 10.0 => println!("Good uniformity - regular hexagons acceptable"),
    v if v < 20.0 => println!("Moderate uniformity - consider higher subdivision"),
    _             => println!("Poor uniformity - increase subdivision level"),
}

// Size range analysis
let size_ratio = stats.max_hexagon_radius / stats.min_hexagon_radius;
println!("Largest hexagon is {:.1}x bigger than smallest", size_ratio);
```

## 🎨 Visualization Tools

### Recommended Software

- **Blender**: Import OBJ files for 3D modeling and rendering
- **MeshLab**: Mesh analysis and processing
- **Unity/Unreal**: Game engine integration
- **ParaView**: Scientific visualization
- **Three.js**: Web-based visualization

### OBJ Export Features

```rust
let obj_content = hexasphere.to_obj();
std::fs::write("hexasphere.obj", obj_content)?;
```

- **Vertex deduplication**: Efficient mesh with shared vertices
- **Mixed polygons**: Hexagons and pentagons in single file
- **1-based indexing**: Standard OBJ format compliance
- **Ready for import**: Works with all major 3D software

## 🔍 Advanced Usage

### Custom Tile Processing

```rust
for (i, tile) in hexasphere.tiles.iter().enumerate() {
    match tile.boundary.len() {
        5 => {
            // Pentagon - handle specially
            println!("Pentagon {} at icosahedral vertex", i);
            let lat_lon = tile.get_lat_lon(hexasphere.radius);
            mark_special_location(lat_lon.lat, lat_lon.lon);
        },
        6 => {
            // Hexagon - regular processing
            let area = tile.get_area();
            let regularity = assess_hexagon_regularity(tile);
            process_hexagon(tile, area, regularity);
        },
        _ => unreachable!("Only pentagons and hexagons exist"),
    }
}
```

### Neighbor Analysis

```rust
// Analyze connectivity
for tile in &hexasphere.tiles {
    println!("Tile has {} neighbors", tile.neighbors.len());
    
    for &neighbor_idx in &tile.neighbors {
        let neighbor = &hexasphere.tiles[neighbor_idx];
        let distance = tile.center_point.distance_to(&neighbor.center_point);
        println!("  Neighbor distance: {:.3}", distance);
    }
}
```

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details on:

- Code style and formatting
- Testing requirements
- Documentation standards
- Pull request process

### Development Setup

```bash
git clone https://github.com/yourusername/geotiles.git
cd geotiles
cargo test
cargo doc --open
```

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Original JavaScript Implementation**: [hexasphere.js](https://github.com/arscan/hexasphere.js) by Rob Scanlon
- **Mathematical Foundation**: Geodesic polyhedra research and Goldberg polyhedra theory
- **Inspiration**: Buckminster Fuller's geodesic dome innovations

## 📚 References

- [Geodesic Polyhedra on Wikipedia](https://en.wikipedia.org/wiki/Geodesic_polyhedron)
- [Goldberg Polyhedra Mathematical Background](https://en.wikipedia.org/wiki/Goldberg_polyhedron)
- [Icosahedral Symmetry and Applications](https://mathworld.wolfram.com/IcosahedralGroup.html)
- [Spherical Trigonometry for Geodesic Calculations](https://mathworld.wolfram.com/SphericalTrigonometry.html)

---

## 📞 Support

- **Documentation**: [docs.rs/geotiles](https://docs.rs/geotiles)
- **Issues**: [GitHub Issues](https://github.com/yourusername/geotiles/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/geotiles/discussions)

Built with ❤️ in Rust
