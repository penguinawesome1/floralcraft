import { type BindGroupLayouts } from "./BindGroupLayouts";

export type PipelineLayouts = {
  gen: GPUPipelineLayout;
  raycast: GPUPipelineLayout;
  render: GPUPipelineLayout;
};

export function createPipelineLayouts(
  device: GPUDevice,
  layouts: BindGroupLayouts,
): PipelineLayouts {
  const gen = device.createPipelineLayout({
    label: "gen pipeline layout",
    bindGroupLayouts: [layouts.atomic_world],
  });

  const raycast = device.createPipelineLayout({
    label: "raycast pipeline layout",
    bindGroupLayouts: [layouts.world, layouts.raycast],
  });

  const render = device.createPipelineLayout({
    label: "render pipeline layout",
    bindGroupLayouts: [layouts.render],
  });

  return { gen, raycast, render };
}
