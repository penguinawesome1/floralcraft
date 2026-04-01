//! A high-performance coordinate projection engine for isometric 2D games.
//!
//! ## The Coordinate System
//! This crate implements a **2:1 Isometric Projection**. In this system:
//! - **X-axis**: Moves tiles down and to the right.
//! - **Y-axis**: Moves tiles down and to the left.
//! - **Z-axis**: Represents vertical height (elevation).
//!
//!
//!
//! ## Why use a Projector?
//! Converting between a player's mouse click (Screen Space) and a voxel's location
//! (Grid Space) requires matrix inversion and scaling. The [`Projector`] struct
//! pre-calculates these transformation matrices at creation, turning complex
//! trigonometric operations into simple matrix-vector multiplications.
//!
//! ## Key Features
//! - **Matrix Caching**: Inverses are calculated once and stored for $O(1)$ lookups.
//! - **Ergonomic API**: Methods accept anything that implements `Into<Vec3>`,
//!   allowing you to pass tuples `(x, y, z)`, `IVec3`, or `Vec3` seamlessly.
//! - **Z-Axis Scaling**: Automatically handles vertical offsets for "tall" blocks or
//!   layered terrain.

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
    /// * `HALF_TW`: The half width of your isometric tile in pixels.
    /// * `HALF_TH`: The half height of your isometric tile in pixels.
    ///
    /// # Examples
    ///
    /// ```
    /// # use spico::Projector;
    /// let proj = Projector::new::<14, 14>();
    /// ```
    pub fn new<const HALF_TW: u32, const HALF_TH: u32>() -> Self {
        let htw = HALF_TW as f32;
        let hth = HALF_TH as f32;
        let mat = Mat2::from_cols(Vec2::new(htw, hth * 0.5), Vec2::new(-htw, hth * 0.5));

        Self {
            proj: mat,
            proj_inv: mat.inverse(),
            z_scale: hth,
            z_scale_inv: 1.0 / hth,
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
    pub fn grid_to_screen(&self, pos: impl Into<Vec3>) -> Vec3 {
        let pos: Vec3 = pos.into();
        let screen_pos: Vec2 = self.proj * pos.truncate();
        screen_pos.extend(pos.z * self.z_scale)
    }

    /// Converts screen positions back to discrete 3D grid coordinates.
    ///
    /// Rounds the result to the nearest integer grid cell.
    pub fn screen_to_grid(&self, pos: impl Into<Vec3>) -> IVec3 {
        let pos = pos.into();
        let grid_pos: Vec2 = self.proj_inv * pos.truncate();
        grid_pos.extend(pos.z * self.z_scale_inv).round().as_ivec3()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tuple_ergonomics() {
        let proj = Projector::new::<16, 8>();
        let screen = proj.grid_to_screen((1.0, 2.0, 3.0));
        let grid = proj.screen_to_grid((64.0, 32.0, 16.0));

        assert!(screen.length() > 0.0);
        assert_eq!(grid.z, 2);
    }

    #[test]
    fn test_round_trip() {
        let proj = Projector::new::<16, 8>();
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
        let proj = Projector::new::<16, 8>();

        let origin = proj.grid_to_screen(Vec3::ZERO);
        assert_eq!(origin, Vec3::ZERO);

        let x_one = proj.grid_to_screen(Vec3::X);
        assert_eq!(x_one.x, 16.0);
        assert_eq!(x_one.y, 4.0);
    }

    #[test]
    fn test_z_height() {
        let proj = Projector::new::<16, 8>();
        let high_block = proj.grid_to_screen(Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(high_block.z, 8.0);
    }
}
