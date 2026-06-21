export type Buffers = {
  world: GPUBuffer;
};

export function createBuffers(device: GPUDevice): Buffers {
  const world = device.createBuffer({
    label: "world buffer",
    size: 128 * 1024 * 1024,
    usage: GPUBufferUsage.STORAGE,
    mappedAtCreation: true,
  });
  new Uint32Array(world.getMappedRange(0, 4)).set([8]);
  world.unmap();
  return { world };
}
