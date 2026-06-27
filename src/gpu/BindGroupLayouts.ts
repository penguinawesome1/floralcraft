export type BindGroupLayouts = {
  read_world: GPUBindGroupLayout;
  read_write_world: GPUBindGroupLayout;
  gen: GPUBindGroupLayout;
  raytrace: GPUBindGroupLayout;
  render: GPUBindGroupLayout;
};

export function createBindGroupLayouts(device: GPUDevice): BindGroupLayouts {
  const read_world = device.createBindGroupLayout({
    label: "read world bind group layout",
    entries: [
      // world
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "read-only-storage" },
      },
    ],
  });

  const read_write_world = device.createBindGroupLayout({
    label: "read write world bind group layout",
    entries: [
      // world
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "storage" },
      },
    ],
  });

  const gen = device.createBindGroupLayout({
    label: "gen bind group layout",
    entries: [
      // chunk presence
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "storage" },
      },
    ],
  });

  const raytrace = device.createBindGroupLayout({
    label: "raytrace bind group layout",
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

  return { read_world, read_write_world, gen, raytrace, render };
}
