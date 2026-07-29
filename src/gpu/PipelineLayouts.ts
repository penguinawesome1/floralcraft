import { type BindGroupLayouts } from "./BindGroupLayouts";

export type PipelineLayouts = {
  free: GPUPipelineLayout;
  clear: GPUPipelineLayout;
  compact: GPUPipelineLayout;
  indirect: GPUPipelineLayout;
  gen: GPUPipelineLayout;
  mip: GPUPipelineLayout;
  raytrace: GPUPipelineLayout;
  modify: GPUPipelineLayout;
  store: GPUPipelineLayout;
  present: GPUPipelineLayout;
};

export function createPipelineLayouts(
  device: GPUDevice,
  layouts: BindGroupLayouts,
): PipelineLayouts {
  const free = device.createPipelineLayout({
    label: "free pipeline layout",
    bindGroupLayouts: [layouts.free],
  });

  const clear = device.createPipelineLayout({
    label: "clear pipeline layout",
    bindGroupLayouts: [layouts.clear],
  });

  const compact = device.createPipelineLayout({
    label: "compact pipeline layout",
    bindGroupLayouts: [layouts.compact],
  });

  const indirect = device.createPipelineLayout({
    label: "indirect pipeline layout",
    bindGroupLayouts: [layouts.indirect],
  });

  const gen = device.createPipelineLayout({
    label: "gen pipeline layout",
    bindGroupLayouts: [layouts.gen],
  });

  const mip = device.createPipelineLayout({
    label: "mip pipeline layout",
    bindGroupLayouts: [layouts.mip],
  });

  const raytrace = device.createPipelineLayout({
    label: "raytrace pipeline layout",
    bindGroupLayouts: [layouts.raytraceStatic, layouts.raytraceDynamic],
  });

  const modify = device.createPipelineLayout({
    label: "modify pipeline layout",
    bindGroupLayouts: [layouts.modify],
  });

  const store = device.createPipelineLayout({
    label: "store pipeline layout",
    bindGroupLayouts: [layouts.store],
  });

  const present = device.createPipelineLayout({
    label: "present pipeline layout",
    bindGroupLayouts: [layouts.present],
  });

  return {
    free,
    clear,
    compact,
    indirect,
    gen,
    mip,
    raytrace,
    modify,
    store,
    present,
  };
}
