export type BindGroupLayouts = {
  compact: GPUBindGroupLayout;
  indirect: GPUBindGroupLayout;
  gen: GPUBindGroupLayout;
  raytraceStatic: GPUBindGroupLayout;
  raytraceDynamic: GPUBindGroupLayout;
  render: GPUBindGroupLayout;
};

export function createBindGroupLayouts(device: GPUDevice): BindGroupLayouts {
  const compact = device.createBindGroupLayout({
    label: "compact bind group layout",
    entries: [
      // gen_flags
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "read-only-storage" },
      },
      // load_list
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "storage" },
      },
    ],
  });

  const indirect = device.createBindGroupLayout({
    label: "indirect bind group layout",
    entries: [
      // indirect_args
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "storage" },
      },
      // load_list
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "read-only-storage" },
      },
    ],
  });

  const gen = device.createBindGroupLayout({
    label: "gen bind group layout",
    entries: [
      // chunk_pool
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "storage" },
      },
      // chunk_index_map
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE,
        storageTexture: {
          access: "write-only",
          format: "r32uint",
          viewDimension: "3d",
        },
      },
      // free_list
      {
        binding: 2,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "storage" },
      },
      // camera
      {
        binding: 3,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "uniform" },
      },
      // load_list
      {
        binding: 4,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "read-only-storage" },
      },
    ],
  });

  const raytraceStatic = device.createBindGroupLayout({
    label: "raytrace static bind group layout",
    entries: [
      // chunk_pool
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "read-only-storage" },
      },
      // chunk_index_map
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE,
        texture: { viewDimension: "3d", sampleType: "uint" },
      },
      // gen_flags
      {
        binding: 2,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "storage" },
      },
    ],
  });

  const raytraceDynamic = device.createBindGroupLayout({
    label: "raytrace dynamic bind group layout",
    entries: [
      // t_output
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

  return { compact, indirect, gen, raytraceStatic, raytraceDynamic, render };
}
