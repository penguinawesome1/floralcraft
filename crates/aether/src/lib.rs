//! The high-level world management engine for the Floralcraft ecosystem.
//!
//! `aether` integrates sparse bit-packed storage ([`chroma`]), coordinate projection ([`spico`]),
//! and asynchronous persistence into a unified API for infinite voxel worlds.
//!
//! ## The Vertical Stack
//! Aether manages data through a tiered hierarchy to optimize for both memory and concurrency:
//! - **World**: Uses a thread-safe `DashMap` to manage horizontal chunk distribution.
//! - **Chunk**: A vertical column of data partitioned into sub-layers.
//! - **Subchunk**: The sparse storage unit that lazily allocates `Section` buffers only when needed.
//!
//! ## Key Features
//! - **Macro-Generated Schemas**: Define voxel properties once; Aether generates the bit-packing logic and accessors.
//! - **Euclidean Coordinate Logic**: Uses `div_euclid` and `rem_euclid` for seamless wrapping across negative chunk boundaries.
//! - **Asynchronous I/O**: Integrated `tokio` and `bincode` support for background chunk persistence.
//!
//! ## Example Usage
//!
//! ```rust
//! use aether::prelude::*;
//!
//! // Define your world structure
//! world! {
//!     ==
//!     [16, 16, 16; 16], // 16x16x16 subchunks, 16 units high
//!     is_solid: bool,
//!     block_id: u16,
//! }
//!
//! fn main() {
//!     let world = World::default();
//!     let pos = BlockPos::new(0, 0, 0);
//!
//!     // Insert a new chunk at the origin
//!     world.insert(&ChunkPos::new(0, 0), None).unwrap(); //
//!
//!     // Access data via generated methods
//!     world.set_is_solid(&pos, true).unwrap(); //
//! }
//! ```

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
    pub use spiral;
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
