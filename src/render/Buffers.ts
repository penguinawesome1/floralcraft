export type Buffers = {
  world: GPUBuffer;
};

export function createBuffers(device: GPUDevice): Buffers {
  const world = device.createBuffer({
    label: "world buffer",
    size: device.limits.maxStorageBufferBindingSize,
    usage: GPUBufferUsage.STORAGE,
  });

  return { world };
}
