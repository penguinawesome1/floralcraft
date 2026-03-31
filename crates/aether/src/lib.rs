pub mod chunk;
pub mod core;
pub mod error;
#[cfg(feature = "persistence")]
pub mod storage;
pub mod subchunk;

#[macro_use]
pub mod world;

pub mod prelude {
    pub use crate::core::{BlockPos, ChunkPos, WorldField};
    pub use crate::error::{AccessError, ChunkAccessError, ChunkOverwriteError, ChunkStoreError};
    pub use crate::world;
    pub use chroma::{BoundsError, Section};
}

#[doc(hidden)]
pub mod __private {
    pub use ahash;
    #[cfg(feature = "persistence")]
    pub use async_trait;
    #[cfg(feature = "persistence")]
    pub use bincode;
    pub use chroma;
    pub use dashmap;
    pub use itertools;
    pub use paste;
    #[cfg(feature = "persistence")]
    pub use serde;
}

#[cfg(feature = "persistence")]
#[macro_export]
macro_rules! derive_persistence {
    ($item:item) => {
        #[derive($crate::__private::serde::Serialize, $crate::__private::serde::Deserialize)]
        $item
    };
}

#[cfg(not(feature = "persistence"))]
#[macro_export]
macro_rules! derive_persistence {
    ($item:item) => {
        $item
    };
}
