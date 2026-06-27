import { type BindGroupLayouts } from "./BindGroupLayouts";

export type PipelineLayouts = {
  gen: GPUPipelineLayout;
  raytrace: GPUPipelineLayout;
  render: GPUPipelineLayout;
};

export function createPipelineLayouts(
  device: GPUDevice,
  layouts: BindGroupLayouts,
): PipelineLayouts {
  const gen = device.createPipelineLayout({
    label: "gen pipeline layout",
    bindGroupLayouts: [layouts.read_write_world, layouts.gen],
  });

  const raytrace = device.createPipelineLayout({
    label: "raytrace pipeline layout",
    bindGroupLayouts: [layouts.read_world, layouts.raytrace],
  });

  const render = device.createPipelineLayout({
    label: "render pipeline layout",
    bindGroupLayouts: [layouts.render],
  });

  return { gen, raytrace, render };
}
