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
  data[0] = 1;
  data[1] = 1;
  world.unmap();
  return { world };
}
