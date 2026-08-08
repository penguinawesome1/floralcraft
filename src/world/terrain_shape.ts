/**
 * CPU mirror of `shaders/terrain_shape.wesl`.
 * Source of truth is terrain_shape.wesl — any change there must be mirrored here.
 * Used for synchronous CPU-side collision queries (can't wait on GPU readback).
 */

import { fbm2d, simplex2d, type Vec2 } from "./noise";

const DOMAIN_WARP_FREQ = 1.0 / 2500.0;
const DOMAIN_WARP_OCTAVES = 4;
const DOMAIN_WARP_OFFSET_Y: Vec2 = [42.5, 11.8];
const DOMAIN_WARP_AMPLITUDE = 40.0;

function domainWarp(pos: Vec2): Vec2 {
  const wx = fbm2d(
    [pos[0] * DOMAIN_WARP_FREQ, pos[1] * DOMAIN_WARP_FREQ],
    DOMAIN_WARP_OCTAVES,
  );
  const wy = fbm2d(
    [
      pos[0] * DOMAIN_WARP_FREQ + DOMAIN_WARP_OFFSET_Y[0],
      pos[1] * DOMAIN_WARP_FREQ + DOMAIN_WARP_OFFSET_Y[1],
    ],
    DOMAIN_WARP_OCTAVES,
  );

  return [
    pos[0] + wx * DOMAIN_WARP_AMPLITUDE,
    pos[1] + wy * DOMAIN_WARP_AMPLITUDE,
  ];
}

const GROUND_DRIFT_OFFSET: Vec2 = [-9173.2, 4408.6];
const GROUND_DRIFT_FREQ = 1.0 / 10000.0;
const GROUND_DRIFT_AMPLITUDE = 500.0;

function groundDrift(pos: Vec2): number {
  return (
    simplex2d([
      (pos[0] + GROUND_DRIFT_OFFSET[0]) * GROUND_DRIFT_FREQ,
      (pos[1] + GROUND_DRIFT_OFFSET[1]) * GROUND_DRIFT_FREQ,
    ]) * GROUND_DRIFT_AMPLITUDE
  );
}

const HILLS_OFFSET: Vec2 = [-9173.2, 4408.6];
const HILLS_FREQ = 1.0 / 600.0;
const HILLS_STEEPNESS = 3.0;
const HILLS_AMPLITUDE = 100.0;

function hills(pos: Vec2): number {
  const noise = simplex2d([
    (pos[0] + HILLS_OFFSET[0]) * HILLS_FREQ,
    (pos[1] + HILLS_OFFSET[1]) * HILLS_FREQ,
  ]);
  const hillCurve = 1.0 / (1.0 + Math.exp(-HILLS_STEEPNESS * noise));
  return hillCurve * HILLS_AMPLITUDE;
}

export function surfaceHeight(pos: readonly [number, number]): number {
  const warped = domainWarp([pos[0], pos[1]]);
  const ground = groundDrift(warped);
  const h = hills(warped);
  return ground + h;
}

export function surfaceDensity(
  pos: readonly [number, number, number],
  terrainHeight: number,
): number {
  return terrainHeight - pos[1];
}

export function caveDensity(
  _pos: readonly [number, number, number],
  _depth: number,
): number {
  return 0.0;
}
