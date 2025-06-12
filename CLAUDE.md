# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build & Development
```bash
cargo build                  # Build debug version
cargo build --release        # Build optimized release version
cargo test                   # Run all tests
cargo test -- --nocapture    # Run tests with println! output visible
cargo doc --no-deps --open   # Generate and open documentation
cargo clippy                 # Run linter
cargo fmt                    # Format code
```

### Testing Specific Areas
```bash
cargo test hexasphere        # Test hexasphere module
cargo test tile              # Test tile module
cargo test geometry          # Test geometry module
```

## Architecture Overview

Geotiles generates geodesic polyhedra (Goldberg polyhedra) by subdividing an icosahedron and projecting it onto a sphere. The resulting structure has mostly hexagonal tiles with exactly 12 pentagons.

### Core Algorithm Flow
1. **Icosahedron Creation**: Start with 12 vertices positioned using golden ratio
2. **Recursive Subdivision**: Each triangle subdivided into 4^n smaller triangles
3. **Sphere Projection**: All vertices normalized to sphere surface
4. **Dual Generation**: Convert vertices to tile centers, faces to boundaries
5. **Neighbor Resolution**: Establish connectivity between adjacent tiles

### Module Structure
- `geometry/`: Basic 3D primitives (Point, Vector3, Face)
- `hexasphere/`: Main hexasphere generation and management
  - Construction algorithm in `hexasphere.rs`
  - Export formats (JSON, OBJ) in `export.rs`
  - Statistical analysis in `statistics.rs`
- `tile/`: Tile representations
  - Basic 2D tiles in `tile.rs`
  - 3D extruded tiles in `thick_tile.rs`
  - Local coordinate systems in `orientation.rs`
- `approximation/`: Regular hexagon approximation parameters
- `utils/`: Mathematical utilities and coordinate conversions

### Key Design Decisions
- **No runtime dependencies**: Core library is dependency-free
- **Immutable design**: Generated hexasphere cannot be modified after creation
- **Builder pattern**: Main constructor takes radius, subdivision level, and tile scale
- **Export abstraction**: Separate module for different export formats

### Performance Notes
- Tile count grows as ~10×4^(n-1) where n is subdivision level
- Level 2 = ~160 tiles, Level 3 = ~640 tiles, Level 4 = ~2560 tiles
- Memory usage scales with tile count
- Point deduplication uses HashMap during construction