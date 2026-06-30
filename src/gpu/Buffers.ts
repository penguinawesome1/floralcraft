import { GEN_SIDE } from "../core/Config";

export type Buffers = {
  world: GPUBuffer;
  gen: GPUBuffer;
};

export function createBuffers(device: GPUDevice): Buffers {
  const world = device.createBuffer({
    label: "world buffer",
    size: 128 * 1024 * 1024,
    usage: GPUBufferUsage.STORAGE,
    mappedAtCreation: true,
  });
  const data = new Uint32Array(world.getMappedRange());
  data.fill(0);
  data[0] = 9;
  data[1] = 1;
  world.unmap();

  const CHUNK_PRESENCE_CAPACITY = Math.ceil(
    (GEN_SIDE * GEN_SIDE * GEN_SIDE) / 32,
  );

  const gen = device.createBuffer({
    label: "gen buffer",
    size: CHUNK_PRESENCE_CAPACITY * 4,
    usage: GPUBufferUsage.STORAGE,
  });

  return { world, gen };
}
