import type { BindGroupLayouts } from "./BindGroupLayouts";
import type { Buffers } from "./Buffers";

export type StaticBindGroups = {
  world: GPUBindGroup;
  atomic_world: GPUBindGroup;
};

export type DynamicBindGroups = {
  raytrace: GPUBindGroup;
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
    layout: layouts.read_world,
    entries: [{ binding: 0, resource: { buffer: buffers.world } }],
  });

  const atomic_world = device.createBindGroup({
    label: "atomic world bind group",
    layout: layouts.read_write_world,
    entries: [{ binding: 0, resource: { buffer: buffers.world } }],
  });

  return { world, atomic_world };
}
