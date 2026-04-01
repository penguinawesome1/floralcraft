# Lattice
[![Rust](https://img.shields.io/badge/language-rust-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> Part of the [Floralcraft](https://github.com/penguinawesome1/floralcraft) engine.

<!-- cargo-rdme start -->

Procedural world generation for the Aether engine.

This crate defines the interface for mapping noise functions and mathematical
distributions to concrete voxel data. It uses a provider-pattern via the
[`Blocks`] trait to remain agnostic of the specific block types used by the game.

### Key Components
- [`Blocks`]: A provider trait that must be implemented by the game's block registry
  to map abstract concepts (like `DIRT` or `GRASS`) to engine-specific types.
- [`BlockGen`]: The core trait for procedural generators. It populates a flattened
  voxel buffer for a given chunk region.
- [`NormalGen`]: A high-performance 2D-heightmap generator powered by `FastNoise2`.

### Coordinate Mapping
Generation occurs in a "Layer-First" layout (`Z -> Y -> X`) to match the
memory alignment of Aether subchunks, ensuring cache-friendly fills during the
world-loading phase.

<!-- cargo-rdme end -->
