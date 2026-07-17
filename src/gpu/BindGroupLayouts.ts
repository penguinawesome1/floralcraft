export type BindGroupLayouts = {
  compact: GPUBindGroupLayout;
  indirect: GPUBindGroupLayout;
  free: GPUBindGroupLayout;
  gen: GPUBindGroupLayout;
  raytraceStatic: GPUBindGroupLayout;
  raytraceDynamic: GPUBindGroupLayout;
  render: GPUBindGroupLayout;
};

function createCompactLayout(device: GPUDevice): GPUBindGroupLayout {
  return device.createBindGroupLayout({
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
}

function createIndirectLayout(device: GPUDevice): GPUBindGroupLayout {
  return device.createBindGroupLayout({
    label: "indirect bind group layout",
    entries: [
      // load_list
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "read-only-storage" },
      },
      // indirect_args
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "storage" },
      },
    ],
  });
}

function createFreeLayout(device: GPUDevice): GPUBindGroupLayout {
  return device.createBindGroupLayout({
    label: "free bind group layout",
    entries: [
      // load_list
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
      // free_list
      {
        binding: 2,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "storage" },
      },
    ],
  });
}

function createGenLayout(device: GPUDevice): GPUBindGroupLayout {
  return device.createBindGroupLayout({
    label: "gen bind group layout",
    entries: [
      // load_list
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "read-only-storage" },
      },
      // camera
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "uniform" },
      },
      // free_list
      {
        binding: 2,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "storage" },
      },
      // chunk_index_map
      {
        binding: 3,
        visibility: GPUShaderStage.COMPUTE,
        storageTexture: {
          access: "write-only",
          format: "r32uint",
          viewDimension: "3d",
        },
      },
      // chunk_pool
      {
        binding: 4,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "storage" },
      },
    ],
  });
}

function createRaytraceStaticLayout(device: GPUDevice): GPUBindGroupLayout {
  return device.createBindGroupLayout({
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
      // skip_mips
      {
        binding: 3,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { type: "read-only-storage" },
      },
    ],
  });
}

function createRaytraceDynamicLayout(device: GPUDevice): GPUBindGroupLayout {
  return device.createBindGroupLayout({
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
}

function createRenderLayout(device: GPUDevice): GPUBindGroupLayout {
  return device.createBindGroupLayout({
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
}

export function createBindGroupLayouts(device: GPUDevice): BindGroupLayouts {
  return {
    compact: createCompactLayout(device),
    indirect: createIndirectLayout(device),
    free: createFreeLayout(device),
    gen: createGenLayout(device),
    raytraceStatic: createRaytraceStaticLayout(device),
    raytraceDynamic: createRaytraceDynamicLayout(device),
    render: createRenderLayout(device),
  };
}
