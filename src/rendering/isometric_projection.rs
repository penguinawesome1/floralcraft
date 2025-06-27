use glam::{ Mat2, Vec2 };
use std::sync::LazyLock;
use crate::game::physics::Position3D;
use crate::terrain::position::BlockPosition;

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
    /// use floralcraft::rendering::IsometricProjection;
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

    /// Converts global positions to their corresponding screen position.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::terrain::BlockPosition;
    /// use floralcraft::game::physics::Position3D;
    /// use floralcraft::rendering::IsometricProjection;
    ///
    /// let proj: IsometricProjection = IsometricProjection::new();
    ///
    /// let pos: BlockPosition = BlockPosition::new(10, 20, 30);
    /// let screen_pos: Position3D = proj.world_to_screen(pos);
    ///
    /// assert!(screen_pos.x != 0.0);
    /// assert!(screen_pos.y != 0.0);
    /// assert!(screen_pos.z != 0.0);
    /// ```
    pub fn world_to_screen(&self, world_pos: BlockPosition) -> Position3D {
        let world_vec_2d: Vec2 = Vec2::new(world_pos.x as f32, world_pos.y as f32);
        let screen_vec_2d: Vec2 = self.iso_matrix_2d * world_vec_2d;

        Position3D {
            x: screen_vec_2d.x,
            y: screen_vec_2d.y,
            z: (world_pos.z as f32) * self.z_scale,
        }
    }

    /// Converts screen positions to their corresponding global position.
    pub fn screen_to_world(&self, screen_pos: Position3D) -> BlockPosition {
        let screen_vec_2d: Vec2 = Vec2::new(screen_pos.x, screen_pos.y);
        let world_vec_2d: Vec2 = self.inv_iso_matrix_2d * screen_vec_2d;

        BlockPosition {
            x: world_vec_2d.x.round() as i32,
            y: world_vec_2d.y.round() as i32,
            z: (screen_pos.z * self.inv_z_scale).round() as i32,
        }
    }
}
