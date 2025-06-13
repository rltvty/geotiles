use geotiles::*;

#[test]
fn test_point_creation() {
    let p = Point::new(1.0, 2.0, 3.0);
    assert_eq!(p.x, 1.0);
    assert_eq!(p.y, 2.0);
    assert_eq!(p.z, 3.0);
}

#[test]
fn test_hexasphere_creation() {
    let hexasphere = Hexasphere::new(10.0, 1, 0.8);
    assert!(hexasphere.tiles.len() > 0);
    assert_eq!(hexasphere.radius, 10.0);
}

#[test]
fn test_point_projection() {
    let mut p = Point::new(1.0, 1.0, 1.0);
    p.project(10.0, 1.0);
    let distance = (p.x.powi(2) + p.y.powi(2) + p.z.powi(2)).sqrt();
    assert!((distance - 10.0).abs() < 0.001);
}

#[test]
fn test_hexagon_stats() {
    let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    let stats = hexasphere.calculate_hexagon_stats();

    assert!(stats.total_hexagons > 0);
    assert_eq!(stats.total_pentagons, 12); // Always 12 pentagons
    assert!(stats.average_hexagon_radius > 0.0);
    assert!(stats.min_hexagon_radius <= stats.average_hexagon_radius);
    assert!(stats.max_hexagon_radius >= stats.average_hexagon_radius);
}

#[test]
fn test_tile_orientation() {
    let hexasphere = Hexasphere::new(10.0, 1, 0.8);
    let orientations = hexasphere.get_tile_orientations();

    assert_eq!(orientations.len(), hexasphere.tiles.len());

    // Check that we have some valid orientations
    let valid_orientations: Vec<_> = orientations.into_iter().flatten().collect();
    assert!(valid_orientations.len() > 0);
}

#[test]
fn test_regular_hexagon_generation() {
    let hexasphere = Hexasphere::new(10.0, 2, 0.8);
    let approximations = hexasphere.get_regular_hexagon_approximations();

    assert!(approximations.len() > 0);

    // Test vertex generation for first hexagon
    if let Some(first_hex) = approximations.first() {
        let vertices = first_hex.generate_vertices();
        assert_eq!(vertices.len(), 6);

        // All vertices should be roughly the same distance from center
        let distances: Vec<f64> = vertices
            .iter()
            .map(|v| first_hex.center.distance_to(v))
            .collect();

        let avg_distance = distances.iter().sum::<f64>() / distances.len() as f64;
        for distance in distances {
            assert!((distance - avg_distance).abs() < 0.1); // Allow small tolerance
        }
    }
}
