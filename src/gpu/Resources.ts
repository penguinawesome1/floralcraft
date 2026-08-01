import {
  GEN_SIDE,
  CHUNK_LEN,
  MAX_CHUNK_BATCH_SIZE,
  MAX_CHUNKS_LOADED,
  MIP_CAPACITY,
  PLAYER_SPAWN,
} from "../core/Config";

export type Resources = {
  player: GPUBuffer;
  unload_params: GPUBuffer;
  gen_flags: GPUBuffer;
  load_list: GPUBuffer;
  indirect_args: GPUBuffer;
  skip_mip: GPUBuffer;
  chunk_pool: GPUBuffer;
  chunk_index_map: GPUTexture;
  free_list: GPUBuffer;
  block_target: GPUBuffer;
  alloc_result: GPUBuffer;
};

function createFreeList(device: GPUDevice): GPUBuffer {
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

  return free_list;
}

export function createResources(device: GPUDevice): Resources {
  const player = device.createBuffer({
    label: "player buffer",
    size: 32,
    usage: GPUBufferUsage.STORAGE,
    mappedAtCreation: true,
  });
  const data = new Float32Array(player.getMappedRange());
  data.set(PLAYER_SPAWN, 0);
  player.unmap();

  const unload_params = device.createBuffer({
    label: "unload_params buffer",
    size: 32,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
  });

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
    size: 24, // xyz, xyz
    usage: GPUBufferUsage.INDIRECT | GPUBufferUsage.STORAGE,
  });

  const skip_mip = device.createBuffer({
    label: "skip_mip buffer",
    size: MIP_CAPACITY * 4,
    usage: GPUBufferUsage.STORAGE,
  });

  const chunk_pool = device.createBuffer({
    label: "chunk_pool buffer",
    size: MAX_CHUNKS_LOADED * CHUNK_LEN * 4,
    usage: GPUBufferUsage.STORAGE,
  });

  const chunk_index_map = device.createTexture({
    label: "chunk_index_map buffer",
    size: [GEN_SIDE, GEN_SIDE, GEN_SIDE],
    dimension: "3d",
    format: "r32uint",
    usage: GPUTextureUsage.STORAGE_BINDING | GPUTextureUsage.TEXTURE_BINDING,
  });

  const block_target = device.createBuffer({
    label: "block_target buffer",
    size: 32,
    usage: GPUBufferUsage.STORAGE,
  });

  const alloc_result = device.createBuffer({
    label: "chunk_alloc_result buffer",
    size: 32,
    usage: GPUBufferUsage.STORAGE,
  });

  return {
    player,
    unload_params,
    gen_flags,
    load_list,
    indirect_args,
    skip_mip,
    chunk_pool,
    chunk_index_map,
    free_list: createFreeList(device),
    block_target,
    alloc_result,
  };
}
