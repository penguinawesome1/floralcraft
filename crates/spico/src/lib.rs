use glam::{IVec3, Mat2, Vec2, Vec3};

/// An isometric projection engine that caches transformation matrices.
///
/// This struct acts as a "Projection Resource" for your game, providing
/// high-performance conversions between grid-space and screen-space.
pub struct Projector {
    proj: Mat2,
    proj_inv: Mat2,
    z_scale: f32,
    z_scale_inv: f32,
}

impl Projector {
    /// Creates a new projection handler based on the pixel dimensions of your tiles.
    ///
    /// This constructor pre-calculates the 2:1 isometric projection matrices and their
    /// inverses to ensure all subsequent coordinate conversions are fast.
    ///
    /// # Type Parameters
    ///
    /// * `TILE_W`: The full width of your isometric tile in pixels.
    /// * `TILE_H`: The full height of your isometric tile in pixels.
    ///
    /// # Examples
    ///
    /// ```
    /// # use spico::Projector;
    /// let proj = Projector::new::<14, 14>();
    /// ```
    pub fn new<const TILE_W: u32, const TILE_H: u32>() -> Self {
        let tw = TILE_W as f32;
        let th = TILE_H as f32;

        let mat = Mat2::from_cols(
            Vec2::new(tw * 0.5, th * 0.25),
            Vec2::new(tw * -0.5, th * 0.25),
        );

        Self {
            proj: mat,
            proj_inv: mat.inverse(),
            z_scale: th * 0.5,
            z_scale_inv: 1.0 / (th * 0.5),
        }
    }

    /// Converts 3D grid positions to their corresponding screen position.
    ///
    /// The Z-axis is scaled to provide vertical height and depth sorting.
    ///
    /// # Examples
    ///
    /// ```
    /// # use glam::IVec3;
    /// # use spico::Projector;
    /// # let proj = Projector::new::<14, 14>();
    /// let pos = IVec3::new(1, 1, 0);
    /// let screen = proj.grid_to_screen(pos.as_vec3());
    /// ```
    #[inline]
    pub fn grid_to_screen(&self, pos: impl Into<Vec3>) -> Vec3 {
        let pos: Vec3 = pos.into();
        let screen_pos: Vec2 = self.proj * pos.truncate();
        screen_pos.extend((pos.z as f32) * self.z_scale)
    }

    /// Converts screen positions back to discrete 3D grid coordinates.
    ///
    /// Rounds the result to the nearest integer grid cell.
    #[inline]
    pub fn screen_to_grid(&self, pos: impl Into<Vec3>) -> IVec3 {
        let pos = pos.into();
        let grid_pos: Vec2 = self.proj_inv * pos.truncate();
        grid_pos.extend(pos.z * self.z_scale_inv).round().as_ivec3()
    }
}
