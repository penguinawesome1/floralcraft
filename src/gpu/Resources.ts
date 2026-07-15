import {
  GEN_SIDE,
  CHUNK_SIDE_SHIFT,
  BITS_PER_ID,
  MAX_CHUNK_BATCH_SIZE,
  MAX_CHUNKS_LOADED,
} from "../core/Config";

export type Resources = {
  gen_flags: GPUBuffer;
  load_list: GPUBuffer;
  indirect_args: GPUBuffer;
  chunk_pool: GPUBuffer;
  chunk_index_map: GPUTexture;
  free_list: GPUBuffer;
};

export function createResources(device: GPUDevice): Resources {
  const CHUNK_LEN = Math.ceil(
    ((1 << CHUNK_SIDE_SHIFT) ** 3 * BITS_PER_ID) / 32,
  );

  const gen_flags = device.createBuffer({
    label: "gen_flags buffer",
    size: Math.ceil(GEN_SIDE ** 3 / 32) * 4,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
  });

  const load_list = device.createBuffer({
    label: "load_list buffer",
    size: (1 + MAX_CHUNK_BATCH_SIZE) * 4,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
  });

  const indirect_args = device.createBuffer({
    label: "indirect_args buffer",
    size: 16,
    usage: GPUBufferUsage.INDIRECT | GPUBufferUsage.STORAGE,
  });

  const chunk_pool = device.createBuffer({
    label: "chunk_pool buffer",
    size: MAX_CHUNKS_LOADED * CHUNK_LEN * 4,
    usage: GPUBufferUsage.STORAGE,
  });

  const chunk_index_map = device.createTexture({
    size: [GEN_SIDE, GEN_SIDE, GEN_SIDE],
    dimension: "3d",
    format: "r32uint",
    usage: GPUTextureUsage.STORAGE_BINDING | GPUTextureUsage.TEXTURE_BINDING,
  });

  const free_list = device.createBuffer({
    label: "free_list buffer",
    size: (1 + MAX_CHUNKS_LOADED) * 4,
    usage: GPUBufferUsage.STORAGE,
    mappedAtCreation: true,
  });

  const data = new Uint32Array(free_list.getMappedRange());
  for (let i = 0; i < MAX_CHUNKS_LOADED; i++) {
    data[1 + i] = i + 1; // reserve 0 for null
  }
  free_list.unmap();

  return {
    gen_flags,
    load_list,
    indirect_args,
    chunk_pool,
    chunk_index_map,
    free_list,
  };
}
