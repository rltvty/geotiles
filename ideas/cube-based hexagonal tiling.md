# Cube-Based Hexagonal Sphere Tiling

## Overview

The cube-based approach for creating hexagonal tilings on a sphere offers a compelling alternative to traditional geodesic/icosahedral methods. Instead of starting with an icosahedron and subdividing triangular faces, this method begins with a cube, applies regular hexagonal grids to each face, and then projects the entire structure onto a sphere.

## Why Use Cube-Based Approach?

### Advantages Over Geodesic Methods
- **More Regular Hexagons**: 90%+ of tiles maintain near-perfect hexagonal shapes
- **Fewer Singularities**: Only 8 problem points (cube corners) vs 12 (icosahedral vertices)
- **Predictable Distortion**: Concentrated at cube edges/corners, uniform elsewhere
- **Easier Implementation**: Simpler coordinate systems and seam handling
- **Better for Regular Tiles**: Ideal when you want consistent hexagon shapes

### Trade-offs
- **Edge Distortion**: Hexagons near cube edges are stretched/compressed
- **Corner Singularities**: 8 points where hexagon regularity breaks down
- **Seam Management**: Requires careful handling of transitions between cube faces

## Algorithm Overview

### Step 1: Create Hexagonal Grids on Cube Faces

```
For each of 6 cube faces:
1. Create a regular hexagonal grid in 2D square space (-1 to +1)
2. Each hexagon has uniform size and perfect regularity
3. Store grid coordinates and hexagon orientations
```

### Step 2: Map to 3D Cube Coordinates

```
For each hexagon center (x, y) on a cube face:
- Face 0 (+X): (1, x, y)
- Face 1 (-X): (-1, -x, y) 
- Face 2 (+Y): (x, 1, -y)
- Face 3 (-Y): (x, -1, y)
- Face 4 (+Z): (x, y, 1)
- Face 5 (-Z): (-x, y, -1)
```

### Step 3: Project to Sphere

Two main projection methods:

#### Simple Normalization (Gnomonic)
```
sphere_position = cube_position.normalize()
```
- Easy to implement
- Significant area distortion (4x size difference)

#### Tangent Adjustment (Better)
```
adjusted_x = (π/4) * atan(cube_x)
adjusted_y = (π/4) * atan(cube_y) 
adjusted_z = (π/4) * atan(cube_z)
sphere_position = (adjusted_x, adjusted_y, adjusted_z).normalize()
```
- More uniform tile sizes
- Slightly more complex to implement

### Step 4: Handle Seams

Connect hexagons across cube face boundaries:
- Identify edge hexagons on adjacent faces
- Merge or align hexagons that should be neighbors
- Handle orientation changes between faces

## Distortion Characteristics

### Minimal Distortion Zones (60% of sphere)
- **Center of each cube face**: Nearly perfect hexagons
- **Area near face centers**: Very uniform sizes and shapes

### Moderate Distortion Zones (30% of sphere)
- **Near cube edges**: Hexagons stretched along edge direction
- **Still recognizably hexagonal**: Good for most applications

### High Distortion Zones (10% of sphere)
- **Near cube corners**: Significant shape distortion
- **8 singular points**: May require special handling (pentagons, different shapes, or exclusion)

## Implementation Considerations

### Grid Resolution
- Higher resolution = more hexagons, finer detail
- Resolution affects both visual quality and performance
- Can be different for each cube face if needed

### Coordinate Systems
Choose appropriate coordinate representation:
- **Cube Face + Local Coordinates**: (face_id, local_x, local_y)
- **Global 3D Coordinates**: Direct sphere positions
- **Cube Coordinates**: Before sphere projection

### Seam Handling Strategies
1. **Perfect Alignment**: Ensure grid points align across faces
2. **Interpolation**: Smooth transitions with slight overlaps
3. **Explicit Connections**: Manually specify neighbor relationships

### Orientation Consistency
Maintain consistent hexagon orientation:
- Define "north" direction for each cube face
- Rotate hexagons to maintain visual flow
- Handle orientation changes at face boundaries

## Comparison with Other Methods

| Method | Regularity | Uniformity | Singularities | Implementation |
|--------|------------|------------|---------------|----------------|
| **Cube-based** | **Excellent** | **Good** | **8 corners** | **Medium** |
| Geodesic/Goldberg | Poor | Good | 12 vertices | Easy |
| HEALPix | Good | Excellent | 8 corners | Hard |
| Stereographic | Perfect (local) | Poor | 2 poles | Easy |

## Use Cases

### Ideal For:
- **Game Development**: Regular tiles for gameplay mechanics
- **Visualization**: Clean, uniform appearance
- **Simulations**: Consistent cell sizes for calculations
- **Procedural Generation**: Predictable grid structure

### Consider Alternatives For:
- **Scientific Mapping**: HEALPix might be better for equal-area requirements
- **Organic Appearance**: Geodesic methods look more "natural"
- **Minimal Implementation**: Simple geodesic subdivision is easier

## Implementation Tips

### Starting Simple
1. Begin with square grids to test cube-to-sphere projection
2. Add hexagonal grids once projection is working
3. Start with simple normalization, add tangent adjustment later

### Optimization Opportunities
- **Pre-compute** projection coordinates for fixed resolutions
- **Cache** neighbor relationships
- **Use instancing** for rendering identical hexagon meshes
- **Level-of-detail**: Different resolutions for different zoom levels

### Debugging Tools
- **Visualize cube before projection**: Verify grid layout
- **Color-code by face**: Identify projection issues
- **Show distortion**: Visualize size/shape variations
- **Highlight seams**: Debug face boundary connections

## Mathematical Details

### Cube Face Normal Vectors
```
Face 0 (+X): (1, 0, 0)
Face 1 (-X): (-1, 0, 0)  
Face 2 (+Y): (0, 1, 0)
Face 3 (-Y): (0, -1, 0)
Face 4 (+Z): (0, 0, 1)
Face 5 (-Z): (0, 0, -1)
```

### Tangent Adjustment Formula
```
For coordinate w in range [-1, 1]:
adjusted_w = (π/4) * atan(w)

Inverse (for texture lookup):
w = tan(adjusted_w * 4/π)
```

### Area Distortion Measurement
```
max_distortion = largest_hexagon_area / smallest_hexagon_area

Simple normalization: ~4x distortion
Tangent adjustment: ~2x distortion
```

## Conclusion

The cube-based hexagonal sphere tiling approach provides an excellent balance of regularity, uniformity, and implementation complexity. While it cannot achieve the perfect equal-area properties of specialized projections like HEALPix, it offers significantly more regular hexagonal shapes than geodesic methods, making it ideal for applications where consistent tile appearance and behavior are important.

The concentrated distortion at cube corners and edges is predictable and manageable, often acceptable for the significant benefits in hexagon regularity across the majority of the sphere surface.

