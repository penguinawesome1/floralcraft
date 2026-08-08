import { mat4, vec3 } from "gl-matrix";
import type { Item } from "../player/Item";

const GEN_SIDE_SHIFT = 8;
const CHUNK_SIDE_SHIFT = 3;
export const GEN_SIDE = 1 << GEN_SIDE_SHIFT;
export const CHUNK_SIDE = 1 << CHUNK_SIDE_SHIFT;
export const BITS_PER_ID = 8; // must be a factor of 32
export const DAY_LENGTH_SECONDS = 600;
export const MAX_CHUNK_BATCH_SIZE = 16384;
export const MAX_CHUNKS_LOADED = 256_000;
export const MIP_CAPACITY = Math.ceil(Math.ceil(GEN_SIDE / 2) ** 3 / 32);
export const CHUNK_LEN = Math.ceil((CHUNK_SIDE ** 3 * BITS_PER_ID) / 32);
export const PLAYER_SPAWN = vec3.fromValues(0, 0, 0);

export const SHADER_CONFIG = {
  GEN_SIDE,
  MAX_CHUNK_BATCH_SIZE,
  CHUNK_SIDE,
  BITS_PER_ID,
} as const;

export type Config = {
  buffer: GPUBuffer;
  uniformData: ArrayBuffer;
  update: (queue: GPUQueue, values: ConfigValues) => void;
};

type ConfigValues = {
  camRotation: mat4;
  camYaw: number;
  maxTraceDist: number;
  timeOfDay: number;
  deltaTime: number;
  pendingAction: number;
  heldItem: Item;
  inputFlags: number;
};

export function packInputFlags(keys: ReadonlySet<string>): number {
  let packed = 0;
  if (keys.has("KeyW") || keys.has("ArrowUp")) packed |= 1;
  if (keys.has("KeyS") || keys.has("ArrowDown")) packed |= 1 << 1;
  if (keys.has("KeyA") || keys.has("ArrowLeft")) packed |= 1 << 2;
  if (keys.has("KeyD") || keys.has("ArrowRight")) packed |= 1 << 3;
  if (keys.has("Space")) packed |= 1 << 4;
  if (keys.has("ShiftLeft")) packed |= 1 << 5;
  return packed;
}

export function createConfig(device: GPUDevice, initial: ConfigValues): Config {
  const uniformData = new ArrayBuffer(96);
  const floatView = new Float32Array(uniformData);
  const uintView = new Uint32Array(uniformData);

  function writeValues(values: ConfigValues) {
    floatView.set(values.camRotation, 0);
    floatView[16] = values.camYaw;
    floatView[17] = values.maxTraceDist;
    floatView[18] = values.timeOfDay;
    floatView[19] = values.deltaTime;
    uintView[20] = values.pendingAction;
    uintView[21] = values.heldItem.kind;
    uintView[22] = values.heldItem.id;
    uintView[23] = values.inputFlags;
  }

  writeValues(initial);

  const buffer = device.createBuffer({
    label: "config buffer",
    size: uniformData.byteLength,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
  });

  device.queue.writeBuffer(buffer, 0, uniformData);

  function update(queue: GPUQueue, values: ConfigValues) {
    writeValues(values);
    queue.writeBuffer(buffer, 0, uniformData);
  }

  return { buffer, uniformData, update };
}
