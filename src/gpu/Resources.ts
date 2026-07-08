import { GEN_SIDE, CHUNK_SIDE_SHIFT, BITS_PER_ID } from "../core/Config";

export type Resources = {
  chunk_pool: GPUBuffer;
  chunk_index_map: GPUTexture;
  free_list: GPUBuffer;
};

export function createResources(device: GPUDevice): Resources {
  const MAX_CHUNKS_LOADED = 32_000;
  const CHUNK_LEN = Math.ceil(
    ((1 << CHUNK_SIDE_SHIFT) ** 3 * BITS_PER_ID) / 32,
  );

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
  data[0] = MAX_CHUNKS_LOADED;
  for (let i = 0; i < MAX_CHUNKS_LOADED; i++) {
    data[1 + i] = i;
  }
  free_list.unmap();

  return { chunk_pool, chunk_index_map, free_list };
}
