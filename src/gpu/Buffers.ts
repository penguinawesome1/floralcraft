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
  const data = new Uint32Array(world.getMappedRange());
  data.fill(0);
  data[0] = 8;
  world.unmap();
  return { world };
}
