# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of Geotiles
- Geodesic polyhedron generation with icosahedral subdivision
- Regular hexagon approximation utilities
- Thick tile support for 3D visualization
- Statistical analysis tools for hexagon uniformity
- JSON and OBJ export functionality
- Comprehensive documentation and examples
- **Face sorting algorithms**: Complete implementation of `sort_faces_around_point()` for proper angular ordering
- **Face centroid cache clearing**: Added `clear_centroid_cache()` method for Face struct
- **Comprehensive test suite**: 68 passing tests covering all functionality
- **Test coverage analysis**: Added cargo-tarpaulin for coverage reporting

### Changed

### Deprecated

### Removed

### Fixed
- **Point.segment() interpolation**: Fixed inverted interpolation logic causing incorrect boundary placement
- **TileOrientation coordinate system**: Fixed default orientation to properly support hexagon generation in XY-plane  
- **Point.subdivide() edge cases**: Added proper handling for zero subdivision count
- **Point.to_lat_lon() coordinate system**: Fixed latitude/longitude conversion for Y-up coordinate system
- **Face vertex projection**: Fixed hexasphere construction to update face vertices after sphere projection
- **Face centroid caching**: Added proper cache invalidation when face vertices are modified
- **Tile boundary generation**: Fixed boundary point placement using correct face centroids
- **Statistical calculations**: Fixed hexagon radius and measurement calculations throughout the system

### Security

## [0.1.0] - 2024-XX-XX

### Added
- Initial public release
