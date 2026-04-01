# Blueprint
[![Rust](https://img.shields.io/badge/language-rust-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> Part of the [Floralcraft](https://github.com/penguinawesome1/floralcraft) engine.

<!-- cargo-rdme start -->

A data-baking pipeline designed for high-performance game engines.
It bridges the gap between human-readable configuration (TOML) and machine-efficient
runtime storage (Bit-packed integers).

## The Strategy: Static Baking
In modern voxel engines, storing block definitions as heavy structs or loading
them via JSON at runtime creates bottlenecks. This crate uses procedural macros
to "bake" your data directly into the application binary's `RODATA` section.

## Key Benefits
- **Zero Startup Overhead**: No parsing occurs at runtime; the data is already
  in memory in its final form.
- **Cache Efficiency**: By bit-packing fields (e.g., packing a `u8` ID and
  a `bool` flag into a single `u16`), we fit more "Definitions" into the CPU L1/L2
  cache, reducing memory latency.
- **Hot-Reloading (Compile-Time)**: Thanks to internal file-tracking,
  changing your TOML data automatically triggers a re-bake during `cargo build`.

## Components
- [`bake_toml`]: The primary macro for generating bit-packed static arrays.

<!-- cargo-rdme end -->
