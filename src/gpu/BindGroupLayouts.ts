export type BindGroupLayouts = {
  world: GPUBindGroupLayout;
  atomic_world: GPUBindGroupLayout;
  raycast: GPUBindGroupLayout;
  render: GPUBindGroupLayout;
};

export function createBindGroupLayouts(device: GPUDevice): BindGroupLayouts {
  const world = device.createBindGroupLayout({
    label: "world bind group layout",
    entries: [
      // world
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "read-only-storage" },
      },
    ],
  });

  const atomic_world = device.createBindGroupLayout({
    label: "atomic world bind group layout",
    entries: [
      // world
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "storage" },
      },
    ],
  });

  const raycast = device.createBindGroupLayout({
    label: "raycast bind group layout",
    entries: [
      // texture_storage_2d
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        storageTexture: { access: "write-only", format: "rgba8unorm" },
      },
      // camera
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "uniform" },
      },
      // config
      {
        binding: 2,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "uniform" },
      },
    ],
  });

  const render = device.createBindGroupLayout({
    label: "render bind group layout",
    entries: [
      // t_canvas
      {
        binding: 0,
        visibility: GPUShaderStage.FRAGMENT,
        texture: {},
      },
      // s_canvas
      {
        binding: 1,
        visibility: GPUShaderStage.FRAGMENT,
        sampler: {},
      },
    ],
  });

  return { world, atomic_world, raycast, render };
}
