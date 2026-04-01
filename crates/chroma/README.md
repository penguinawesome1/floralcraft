# Chroma
[![Rust](https://img.shields.io/badge/language-rust-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> Part of the [Floralcraft](https://github.com/penguinawesome1/floralcraft) engine.

<!-- cargo-rdme start -->

A memory-efficient, palette-based voxel storage engine designed for sparse 3D grids.

`Section<T, W, H, D>` provides a bit-packed container where items are stored as
indices into a dynamic palette. This allows for massive memory savings in
environments with limited unique voxel types (e.g., a sky section containing
only `Air` and `Cloud`).

### Key Features
- **Generic Dimensions**: Uses Rust's `const` generics for compile-time bounds and array sizing.
- **Dynamic Repacking**: Automatically increases bit-width (1, 2, 4, 8, etc.) as the
  palette grows to accommodate more unique items.
- **Performance**: Optimized bit-shuffling logic allows for `u64` word-aligned reads
  and writes, even when an item straddles a word boundary.

### Memory Footprint
The theoretical size of a `16x16x16` section at 4 bits per item is roughly **2,048 bytes**,
compared to **16,384 bytes** if stored as raw `u32` values.

### Example
```rust
use glam::IVec3;
use chroma::Section;

// Create a 16x16x16 section starting with 2 bits per item
let mut section: Section<u32, 16, 16, 16> = Section::new(2);

// Setting an item that exceeds the current 2-bit (4 unique items) limit
// will trigger an internal `repack()` to increase bit-depth.
section.set(IVec3::new(0, 5, 0), 99).unwrap();

assert_eq!(section.get((0, 5, 0)), Some(99));
```

<!-- cargo-rdme end -->
