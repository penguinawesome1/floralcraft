//! Procedural world generation for the Aether engine.
//!
//! This crate defines the interface for mapping noise functions and mathematical
//! distributions to concrete voxel data. It uses a provider-pattern via the
//! [`Blocks`] trait to remain agnostic of the specific block types used by the game.
//!
//! ## Key Components
//! - [`Blocks`]: A provider trait that must be implemented by the game's block registry
//!   to map abstract concepts (like `DIRT` or `GRASS`) to engine-specific types.
//! - [`BlockGen`]: The core trait for procedural generators. It populates a flattened
//!   voxel buffer for a given chunk region.
//! - [`NormalGen`]: A high-performance 2D-heightmap generator powered by `FastNoise2`.
//!
//! ## Coordinate Mapping
//! Generation occurs in a "Layer-First" layout (`Z -> Y -> X`) to match the
//! memory alignment of Aether subchunks, ensuring cache-friendly fills during the
//! world-loading phase.

use aether::prelude::BlockPos;
use fastnoise2::SafeNode;
use glam::IVec3;
use std::fmt::Debug;

pub trait Blocks: Send + Sync + 'static {
    type T: Clone + Copy + PartialEq + Default + Send + Sync + Debug + 'static;
    const AIR: Self::T;
    const GRASS: Self::T;
    const DIRT: Self::T;
    const STONE: Self::T;
    const ROSE: Self::T;
    const DANDELION: Self::T;
    const BEDROCK: Self::T;
}

pub trait BlockGen<B: Blocks, const L: usize, const V: usize>: Send + Sync + 'static {
    fn choose_blocks(&self, start_pos: BlockPos, out: &mut [B::T; V]);
}

use std::marker::PhantomData;

pub struct FlatGen<B: Blocks> {
    _marker: PhantomData<B>,
}

impl<B: Blocks> Default for FlatGen<B> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<B: Blocks, const L: usize, const V: usize> BlockGen<B, L, V> for FlatGen<B> {
    fn choose_blocks(&self, start_pos: IVec3, out: &mut [B::T; V]) {
        debug_assert_eq!(V, L * L * L, "Volume must be L^3");

        for z in 0..L {
            let world_z = start_pos.z + z as i32;

            let block = match world_z {
                0 => B::BEDROCK,
                1..=3 => B::DIRT,
                4 => B::GRASS,
                _ => B::AIR,
            };

            let start = z * L * L;
            out[start..start + L * L].fill(block);
        }
    }
}

pub struct BlockGenParams {
    pub seed: i32,
    pub scale: f32,
    pub min_depth: u32,
    pub max_depth: u32,
    pub dirt_depth: u32,
}

pub struct NormalGen<B: Blocks> {
    node: SafeNode,
    params: BlockGenParams,
    _marker: PhantomData<B>,
}

impl<B: Blocks> NormalGen<B> {
    pub fn new(noise_profile: &str, params: BlockGenParams) -> Self {
        Self {
            node: SafeNode::from_encoded_node_tree(noise_profile)
                .expect("Invalid node tree string"),
            params,
            _marker: PhantomData,
        }
    }
}

impl<B: Blocks, const L: usize, const V: usize> BlockGen<B, L, V> for NormalGen<B> {
    fn choose_blocks(&self, start_pos: IVec3, out: &mut [B::T; V]) {
        debug_assert_eq!(V, L * L * L, "Volume must be L^3");

        let BlockGenParams {
            seed,
            scale,
            min_depth,
            max_depth,
            dirt_depth,
            ..
        } = self.params;

        let mut height_map = vec![0.0; L * L];

        self.node.gen_uniform_grid_2d(
            &mut height_map,
            start_pos.x as f32 * scale,
            start_pos.y as f32 * scale,
            L as i32,
            L as i32,
            scale,
            scale,
            seed,
        );

        let height_range = (max_depth - min_depth) as f32;

        let mut surface_zs = vec![0i32; L * L];
        for i in 0..(L * L) {
            let norm = (height_map[i] + 1.0) * 0.5;
            surface_zs[i] = (min_depth as f32 + height_range * norm) as i32;
        }

        for z in 0..L {
            let world_z = start_pos.z + z as i32;
            let layer_offset = z * L * L;

            for i in 0..(L * L) {
                let surface_z = surface_zs[i];
                let dirt_z = surface_z - dirt_depth as i32;

                out[layer_offset + i] = match world_z {
                    0 => B::BEDROCK,
                    z if z > surface_z => B::AIR,
                    z if z == surface_z => B::GRASS,
                    z if z >= dirt_z => B::DIRT,
                    _ => B::STONE,
                };
            }
        }
    }
}
