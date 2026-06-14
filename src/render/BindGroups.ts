import type { BindGroupLayouts } from "./BindGroupLayouts";
import type { Buffers } from "./Buffers";

export type StaticBindGroups = {
  world: GPUBindGroup;
};

export type DynamicBindGroups = {
  raycast: GPUBindGroup;
  render: GPUBindGroup;
};

export type BindGroups = StaticBindGroups & DynamicBindGroups;

export function createStaticBindGroups(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  buffers: Buffers,
): StaticBindGroups {
  const world = device.createBindGroup({
    label: "world bind group",
    layout: layouts.world,
    entries: [
      { binding: 0, resource: { buffer: buffers.world } },
    ],
  });

  return { world };
}
