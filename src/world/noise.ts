/**
 * CPU mirror of `shaders/noise.wesl`.
 * Source of truth is noise.wesl — any change there must be mirrored here.
 * Used for synchronous CPU-side collision queries (can't wait on GPU readback).
 */

export type Vec2 = readonly [number, number];
export type Vec3 = readonly [number, number, number];

function mulU32(a: number, b: number): number {
  return Math.imul(a, b) >>> 0;
}

function addU32(a: number, b: number): number {
  return (a + b) >>> 0;
}

function xorU32(a: number, b: number): number {
  return (a ^ b) >>> 0;
}

function shrU32(a: number, shift: number): number {
  return a >>> shift;
}

function hash2d(p: Vec2): Vec2 {
  let vx = Math.trunc(p[0]) >>> 0;
  let vy = Math.trunc(p[1]) >>> 0;

  vx = addU32(mulU32(vx, 1664525), 1013904223);
  vy = addU32(mulU32(vy, 1664525), 1013904223);

  vx = addU32(vx, mulU32(vy, 1664525));
  vy = addU32(vy, mulU32(vx, 1664525));

  vx = xorU32(vx, shrU32(vx, 16));
  vy = xorU32(vy, shrU32(vy, 16));

  vx = addU32(vx, mulU32(vy, 1664525));
  vy = addU32(vy, mulU32(vx, 1664525));

  vx = xorU32(vx, shrU32(vx, 16));
  vy = xorU32(vy, shrU32(vy, 16));

  return [
    -1.0 + 2.0 * (vx * (1.0 / 4294967295.0)),
    -1.0 + 2.0 * (vy * (1.0 / 4294967295.0)),
  ];
}

function hash3d(p: Vec3): Vec3 {
  let vx = Math.trunc(p[0]) >>> 0;
  let vy = Math.trunc(p[1]) >>> 0;
  let vz = Math.trunc(p[2]) >>> 0;

  vx = addU32(mulU32(vx, 1664525), 1013904223);
  vy = addU32(mulU32(vy, 1664525), 1013904223);
  vz = addU32(mulU32(vz, 1664525), 1013904223);

  vx = addU32(vx, mulU32(vy, vz));
  vy = addU32(vy, mulU32(vz, vx));
  vz = addU32(vz, mulU32(vx, vy));

  vx = xorU32(vx, shrU32(vx, 16));
  vy = xorU32(vy, shrU32(vy, 16));
  vz = xorU32(vz, shrU32(vz, 16));

  vx = addU32(vx, mulU32(vy, vz));
  vy = addU32(vy, mulU32(vz, vx));
  vz = addU32(vz, mulU32(vx, vy));

  vx = xorU32(vx, shrU32(vx, 16));
  vy = xorU32(vy, shrU32(vy, 16));
  vz = xorU32(vz, shrU32(vz, 16));

  return [
    -1.0 + 2.0 * (vx * (1.0 / 4294967295.0)),
    -1.0 + 2.0 * (vy * (1.0 / 4294967295.0)),
    -1.0 + 2.0 * (vz * (1.0 / 4294967295.0)),
  ];
}

function dot2(a: Vec2, b: Vec2): number {
  return a[0] * b[0] + a[1] * b[1];
}

function dot3(a: Vec3, b: Vec3): number {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

export function simplex2d(p: Vec2): number {
  const K1 = 0.366025404;
  const K2 = 0.211324865;

  const sum = p[0] + p[1];
  const i: Vec2 = [Math.floor(p[0] + sum * K1), Math.floor(p[1] + sum * K1)];

  const iSum = i[0] + i[1];
  const a: Vec2 = [p[0] - i[0] + iSum * K2, p[1] - i[1] + iSum * K2];

  const o: Vec2 = a[0] < a[1] ? [0.0, 1.0] : [1.0, 0.0];

  const b: Vec2 = [a[0] - o[0] + K2, a[1] - o[1] + K2];
  const c: Vec2 = [a[0] - 1.0 + 2.0 * K2, a[1] - 1.0 + 2.0 * K2];

  const h0 = Math.max(0.5 - dot2(a, a), 0.0);
  const h1 = Math.max(0.5 - dot2(b, b), 0.0);
  const h2 = Math.max(0.5 - dot2(c, c), 0.0);

  const g0 = dot2(a, hash2d([i[0] + 0.0, i[1] + 0.0]));
  const g1 = dot2(b, hash2d([i[0] + o[0], i[1] + o[1]]));
  const g2 = dot2(c, hash2d([i[0] + 1.0, i[1] + 1.0]));

  const n0 = h0 * h0 * h0 * h0 * g0;
  const n1 = h1 * h1 * h1 * h1 * g1;
  const n2 = h2 * h2 * h2 * h2 * g2;

  return (n0 + n1 + n2) * 70.0;
}

export function simplex3d(p: Vec3): number {
  const F3 = 1.0 / 3.0;
  const G3 = 1.0 / 6.0;

  const s = (p[0] + p[1] + p[2]) * F3;
  const i: Vec3 = [
    Math.floor(p[0] + s),
    Math.floor(p[1] + s),
    Math.floor(p[2] + s),
  ];
  const t = (i[0] + i[1] + i[2]) * G3;
  const a: Vec3 = [p[0] - i[0] + t, p[1] - i[1] + t, p[2] - i[2] + t];

  let i1: Vec3;
  let i2: Vec3;

  if (a[0] >= a[1]) {
    if (a[1] >= a[2]) {
      i1 = [1.0, 0.0, 0.0];
      i2 = [1.0, 1.0, 0.0];
    } else if (a[0] >= a[2]) {
      i1 = [1.0, 0.0, 0.0];
      i2 = [1.0, 0.0, 1.0];
    } else {
      i1 = [0.0, 0.0, 1.0];
      i2 = [1.0, 0.0, 1.0];
    }
  } else {
    if (a[1] < a[2]) {
      i1 = [0.0, 0.0, 1.0];
      i2 = [0.0, 1.0, 1.0];
    } else if (a[0] < a[2]) {
      i1 = [0.0, 1.0, 0.0];
      i2 = [0.0, 1.0, 1.0];
    } else {
      i1 = [0.0, 1.0, 0.0];
      i2 = [1.0, 1.0, 0.0];
    }
  }

  const b: Vec3 = [a[0] - i1[0] + G3, a[1] - i1[1] + G3, a[2] - i1[2] + G3];
  const c: Vec3 = [
    a[0] - i2[0] + 2.0 * G3,
    a[1] - i2[1] + 2.0 * G3,
    a[2] - i2[2] + 2.0 * G3,
  ];
  const d: Vec3 = [
    a[0] - 1.0 + 3.0 * G3,
    a[1] - 1.0 + 3.0 * G3,
    a[2] - 1.0 + 3.0 * G3,
  ];

  const h0 = Math.max(0.6 - dot3(a, a), 0.0);
  const h1 = Math.max(0.6 - dot3(b, b), 0.0);
  const h2 = Math.max(0.6 - dot3(c, c), 0.0);
  const h3 = Math.max(0.6 - dot3(d, d), 0.0);

  const g0 = dot3(a, hash3d(i));
  const g1 = dot3(b, hash3d([i[0] + i1[0], i[1] + i1[1], i[2] + i1[2]]));
  const g2 = dot3(c, hash3d([i[0] + i2[0], i[1] + i2[1], i[2] + i2[2]]));
  const g3 = dot3(d, hash3d([i[0] + 1.0, i[1] + 1.0, i[2] + 1.0]));

  const n0 = h0 * h0 * h0 * h0 * g0;
  const n1 = h1 * h1 * h1 * h1 * g1;
  const n2 = h2 * h2 * h2 * h2 * g2;
  const n3 = h3 * h3 * h3 * h3 * g3;

  return (n0 + n1 + n2 + n3) * 32.0;
}

export function fbm2d(pos: Vec2, octaves: number): number {
  let value = 0.0;
  let amplitude = 0.5;
  let frequency = 1.0;
  let maxValue = 0.0;

  for (let i = 0; i < octaves; i++) {
    value += simplex2d([pos[0] * frequency, pos[1] * frequency]) * amplitude;
    maxValue += amplitude;
    amplitude *= 0.5;
    frequency *= 2.0;
  }

  return value / maxValue;
}

export function fbm3d(pos: Vec3, octaves: number): number {
  let value = 0.0;
  let amplitude = 0.5;
  let frequency = 1.0;
  let maxValue = 0.0;

  for (let i = 0; i < octaves; i++) {
    value +=
      simplex3d([pos[0] * frequency, pos[1] * frequency, pos[2] * frequency]) *
      amplitude;
    maxValue += amplitude;
    amplitude *= 0.5;
    frequency *= 2.0;
  }

  return value / maxValue;
}

export function ridged2d(pos: Vec2, octaves: number): number {
  let value = 0.0;
  let amplitude = 0.5;
  let frequency = 1.0;
  let maxValue = 0.0;

  for (let i = 0; i < octaves; i++) {
    const noise = simplex2d([pos[0] * frequency, pos[1] * frequency]);
    const shaped = Math.pow(1.0 - Math.abs(noise), 2.0);
    value += shaped * amplitude;
    maxValue += amplitude;
    amplitude *= 0.5;
    frequency *= 2.0;
  }

  return value / maxValue;
}
