# Spico
[![Rust](https://img.shields.io/badge/language-rust-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> Part of the [Floralcraft](https://github.com/penguinawesome1/floralcraft) engine.

<!-- cargo-rdme start -->

A high-performance coordinate projection engine for isometric 2D games.

### The Coordinate System
This crate implements a **2:1 Isometric Projection**. In this system:
- **X-axis**: Moves tiles down and to the right.
- **Y-axis**: Moves tiles down and to the left.
- **Z-axis**: Represents vertical height (elevation).



### Why use a Projector?
Converting between a player's mouse click (Screen Space) and a voxel's location
(Grid Space) requires matrix inversion and scaling. The [`Projector`] struct
pre-calculates these transformation matrices at creation, turning complex
trigonometric operations into simple matrix-vector multiplications.

### Key Features
- **Matrix Caching**: Inverses are calculated once and stored for $O(1)$ lookups.
- **Ergonomic API**: Methods accept anything that implements `Into<Vec3>`,
  allowing you to pass tuples `(x, y, z)`, `IVec3`, or `Vec3` seamlessly.
- **Z-Axis Scaling**: Automatically handles vertical offsets for "tall" blocks or
  layered terrain.

<!-- cargo-rdme end -->
