import { type BindGroupLayouts } from "./BindGroupLayouts";

export type PipelineLayouts = {
  compact: GPUPipelineLayout;
  indirect: GPUPipelineLayout;
  free: GPUPipelineLayout;
  gen: GPUPipelineLayout;
  mip: GPUPipelineLayout;
  raytrace: GPUPipelineLayout;
  present: GPUPipelineLayout;
};

export function createPipelineLayouts(
  device: GPUDevice,
  layouts: BindGroupLayouts,
): PipelineLayouts {
  const compact = device.createPipelineLayout({
    label: "compact pipeline layout",
    bindGroupLayouts: [layouts.compact],
  });

  const indirect = device.createPipelineLayout({
    label: "indirect pipeline layout",
    bindGroupLayouts: [layouts.indirect],
  });

  const free = device.createPipelineLayout({
    label: "free pipeline layout",
    bindGroupLayouts: [layouts.free],
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

  const present = device.createPipelineLayout({
    label: "present pipeline layout",
    bindGroupLayouts: [layouts.present],
  });

  return { compact, indirect, free, gen, mip, raytrace, present };
}
