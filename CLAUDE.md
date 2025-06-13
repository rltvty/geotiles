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
cargo test utils             # Test utility functions
cargo test approximation     # Test regular hexagon approximation
```

### Coverage Analysis
```bash
cargo install cargo-tarpaulin  # Install coverage tool (one time)
cargo tarpaulin --out Stdout   # Generate test coverage report
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
  - Complete geometric operations with proper coordinate systems
  - Point interpolation, subdivision, and sphere projection
  - Face centroid calculation and adjacency detection
- `hexasphere/`: Main hexasphere generation and management
  - Construction algorithm in `core.rs` (renamed from hexasphere.rs)
  - Export formats (JSON, OBJ) in `export.rs`
  - Statistical analysis in `statistics.rs`
- `tile/`: Tile representations
  - Basic 2D tiles in `core.rs` (renamed from tile.rs)
  - 3D extruded tiles in `thick_tile.rs`
  - Local coordinate systems in `orientation.rs`
- `approximation/`: Regular hexagon approximation parameters
- `utils/`: Mathematical utilities and coordinate conversions
  - Face sorting algorithms for proper tile boundary ordering
  - Coordinate system conversions and geometric calculations

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

## Implementation Quality

### Test Coverage
- **68 passing tests** covering all major functionality
- **Comprehensive test suite** including unit tests, integration tests, and doc tests
- **Statistical validation** of hexagon properties and geometric correctness
- **Edge case handling** for boundary conditions and degenerate inputs

### Code Quality
- **Fully implemented algorithms**: All core functions are complete and working
- **Proper coordinate systems**: Y-up coordinate system with correct transformations
- **Robust geometry**: Face sorting, angular ordering, and boundary generation
- **Statistical analysis**: Accurate measurements of tile properties and variations

### Known Working Features
- ✅ Icosahedron generation and subdivision
- ✅ Sphere projection with proper vertex updating  
- ✅ Tile boundary generation and scaling
- ✅ Face centroid calculation and caching
- ✅ Angular face sorting around vertices
- ✅ Regular hexagon approximation
- ✅ Export to JSON and OBJ formats
- ✅ Statistical analysis of hexagon properties
- ✅ Coordinate system transformations