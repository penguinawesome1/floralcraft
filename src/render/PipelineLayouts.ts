import { type BindGroupLayouts } from "./BindGroupLayouts";

export type PipelineLayouts = {
  raycast: GPUPipelineLayout;
  render: GPUPipelineLayout;
};

export function createPipelineLayouts(
  device: GPUDevice,
  layouts: BindGroupLayouts,
): PipelineLayouts {
  const raycast = device.createPipelineLayout({
    label: "raycast pipeline layout",
    bindGroupLayouts: [layouts.world, layouts.raycast],
  });

  const render = device.createPipelineLayout({
    label: "render pipeline layout",
    bindGroupLayouts: [layouts.render],
  });

  return { raycast, render };
}
