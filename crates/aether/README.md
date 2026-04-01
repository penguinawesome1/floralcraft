# Aether
[![Rust](https://img.shields.io/badge/language-rust-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> Part of the [Floralcraft](https://github.com/penguinawesome1/floralcraft) engine.

<!-- cargo-rdme start -->

The high-level world management engine for the Floralcraft ecosystem.

`aether` integrates sparse bit-packed storage ([`chroma`]), coordinate projection ([`spico`]),
and asynchronous persistence into a unified API for infinite voxel worlds.

### The Vertical Stack
Aether manages data through a tiered hierarchy to optimize for both memory and concurrency:
- **World**: Uses a thread-safe `DashMap` to manage horizontal chunk distribution.
- **Chunk**: A vertical column of data partitioned into sub-layers.
- **Subchunk**: The sparse storage unit that lazily allocates `Section` buffers only when needed.

### Key Features
- **Macro-Generated Schemas**: Define voxel properties once; Aether generates the bit-packing logic and accessors.
- **Euclidean Coordinate Logic**: Uses `div_euclid` and `rem_euclid` for seamless wrapping across negative chunk boundaries.
- **Asynchronous I/O**: Integrated `tokio` and `bincode` support for background chunk persistence.

### Example Usage

```rust
use aether::prelude::*;

// Define your world structure
world! {
    ==
    [16, 16, 16; 16], // 16x16x16 subchunks, 16 units high
    is_solid: bool,
    block_id: u16,
}

fn main() {
    let world = World::default();
    let pos = BlockPos::new(0, 0, 0);

    // Insert a new chunk at the origin
    world.insert(&ChunkPos::new(0, 0), None).unwrap();

    // Access data via generated methods
    world.set_is_solid(&pos, true).unwrap();
}
```

<!-- cargo-rdme end -->
