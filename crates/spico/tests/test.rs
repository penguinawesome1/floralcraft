use glam::{IVec3, Vec3};
use spico::Projector;

#[test]
fn test_round_trip() {
    let proj = Projector::new::<32, 16>();
    let original_grid = IVec3::new(5, -3, 2);
    let screen = proj.grid_to_screen(original_grid.as_vec3());
    let result_grid = proj.screen_to_grid(screen);

    assert_eq!(
        original_grid, result_grid,
        "The round-trip conversion should be lossless."
    );
}

#[test]
fn test_specific_mapping() {
    let proj = Projector::new::<32, 16>();

    let origin = proj.grid_to_screen(Vec3::ZERO);
    assert_eq!(origin, Vec3::ZERO);

    let x_one = proj.grid_to_screen(Vec3::X);
    assert_eq!(x_one.x, 16.0);
    assert_eq!(x_one.y, 4.0);
}

#[test]
fn test_z_height() {
    let proj = Projector::new::<32, 16>();
    let high_block = proj.grid_to_screen(Vec3::new(0.0, 0.0, 1.0));
    assert_eq!(high_block.z, 8.0);
}
