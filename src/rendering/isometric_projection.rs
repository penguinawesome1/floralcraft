use glam::{ Mat2, Vec2, Vec3, IVec3 };
use std::sync::LazyLock;

const TILE_WIDTH: u32 = 28;
const TILE_HEIGHT: u32 = 28;
const HALF_TILE_WIDTH: f32 = (TILE_WIDTH as f32) / 2.0;
const HALF_TILE_HEIGHT: f32 = (TILE_HEIGHT as f32) / 2.0;

pub static PROJECTION: LazyLock<IsometricProjection> = LazyLock::new(|| {
    IsometricProjection::new()
});

pub struct IsometricProjection {
    /// The 2x2 matrix for the XY part of the isometric projection.
    iso_matrix_2d: Mat2,
    /// The inverse of the 2x2 matrix.
    inv_iso_matrix_2d: Mat2,
    /// Scalar for Z-axis scaling (world_z to screen_z).
    z_scale: f32,
    /// Inverse scalar for Z-axis scaling (screen_z to world_z).
    inv_z_scale: f32,
}

impl IsometricProjection {
    /// Create a new projection struct to convert between world and screen.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::rendering::isometric_projection::IsometricProjection;
    ///
    /// let proj: IsometricProjection = IsometricProjection::new();
    /// ```
    pub fn new() -> Self {
        let iso_matrix_2d = Mat2::from_cols(
            Vec2::new(1.0 * HALF_TILE_WIDTH, 0.5 * HALF_TILE_HEIGHT),
            Vec2::new(-1.0 * HALF_TILE_WIDTH, 0.5 * HALF_TILE_HEIGHT)
        );

        let inv_iso_matrix_2d = iso_matrix_2d.inverse();

        let z_scale = HALF_TILE_HEIGHT;
        let inv_z_scale = 1.0 / z_scale;

        Self {
            iso_matrix_2d,
            inv_iso_matrix_2d,
            z_scale,
            inv_z_scale,
        }
    }

    /// Converts 3d grid positions to their corresponding screen position.
    ///
    /// # Examples
    ///
    /// ```
    /// use glam::{ IVec3, Vec3 };
    /// use floralcraft::rendering::isometric_projection::IsometricProjection;
    ///
    /// let proj: IsometricProjection = IsometricProjection::new();
    ///
    /// let pos: IVec3 = IVec3::new(10, 20, 30);
    /// let screen_pos: Vec3 = proj.world_to_screen(pos);
    ///
    /// assert!(screen_pos.x != 0.0);
    /// assert!(screen_pos.y != 0.0);
    /// assert!(screen_pos.z != 0.0);
    /// ```
    pub fn world_to_screen(&self, world_pos: IVec3) -> Vec3 {
        let world_vec_2d: Vec2 = Vec2::new(world_pos.x as f32, world_pos.y as f32);
        let screen_vec_2d: Vec2 = self.iso_matrix_2d * world_vec_2d;

        Vec3::new(screen_vec_2d.x, screen_vec_2d.y, (world_pos.z as f32) * self.z_scale)
    }

    /// Converts screen positions to their corresponding 3d grid positions.
    pub fn screen_to_world(&self, screen_pos: Vec3) -> IVec3 {
        let screen_vec_2d: Vec2 = Vec2::new(screen_pos.x, screen_pos.y);
        let world_vec_2d: Vec2 = self.inv_iso_matrix_2d * screen_vec_2d;

        IVec3::new(
            world_vec_2d.x.round() as i32,
            world_vec_2d.y.round() as i32,
            (screen_pos.z * self.inv_z_scale).round() as i32
        )
    }
}
