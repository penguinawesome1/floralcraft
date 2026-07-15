import { type BindGroupLayouts } from "./BindGroupLayouts";

export type PipelineLayouts = {
  compact: GPUPipelineLayout;
  indirect: GPUPipelineLayout;
  gen: GPUPipelineLayout;
  raytrace: GPUPipelineLayout;
  render: GPUPipelineLayout;
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

  const gen = device.createPipelineLayout({
    label: "gen pipeline layout",
    bindGroupLayouts: [layouts.gen],
  });

  const raytrace = device.createPipelineLayout({
    label: "raytrace pipeline layout",
    bindGroupLayouts: [layouts.raytraceStatic, layouts.raytraceDynamic],
  });

  const render = device.createPipelineLayout({
    label: "render pipeline layout",
    bindGroupLayouts: [layouts.render],
  });

  return { compact, indirect, gen, raytrace, render };
}
