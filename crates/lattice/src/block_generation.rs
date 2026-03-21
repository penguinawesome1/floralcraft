// use glam::IVec3;

// pub struct GenParams;

// pub trait BlockGenerator<T>: Send + Sync + 'static {
//     fn choose_block(&self, pos: IVec3, params: &GenParams) -> T;
// }

// #[derive(Clone)]
// pub struct FlatGenerator;

// impl BlockGenerator for FlatGenerator {
//     fn choose_block(&self, pos: BlockPos, _params: &WorldGeneration) -> SnugType {
//         match pos.z {
//             0 => BEDROCK,
//             1..=3 => DIRT,
//             4 => GRASS,
//             _ => AIR,
//         }
//     }
// }
