use glam::IVec3;
use std::fmt::Debug;

pub type BlockPos = glam::IVec3;

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

pub struct BlockGenParams;

pub trait BlockGen<B: Blocks>: Send + Sync + 'static {
    fn choose_block(&self, pos: IVec3, params: &BlockGenParams) -> B::T;
}

use std::marker::PhantomData;

pub struct FlatGen<B: Blocks> {
    _marker: PhantomData<B>,
}

impl<B: Blocks> FlatGen<B> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<B: Blocks> BlockGen<B> for FlatGen<B> {
    fn choose_block(&self, pos: BlockPos, _: &BlockGenParams) -> B::T {
        match pos.z {
            0 => B::BEDROCK,
            1..=3 => B::DIRT,
            4 => B::GRASS,
            _ => B::AIR,
        }
    }
}
