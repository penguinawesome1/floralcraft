import type { BindGroupLayouts } from "./BindGroupLayouts";
import type { Buffers } from "./Buffers";

export type StaticBindGroups = {
  read_world: GPUBindGroup;
  read_write_world: GPUBindGroup;
  gen: GPUBindGroup;
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
  const read_world = device.createBindGroup({
    label: "read world bind group",
    layout: layouts.read_world,
    entries: [{ binding: 0, resource: { buffer: buffers.world } }],
  });

  const read_write_world = device.createBindGroup({
    label: "read write world bind group",
    layout: layouts.read_write_world,
    entries: [{ binding: 0, resource: { buffer: buffers.world } }],
  });

  const gen = device.createBindGroup({
    label: "gen bind group",
    layout: layouts.gen,
    entries: [{ binding: 0, resource: { buffer: buffers.gen } }],
  });

  return { read_world, read_write_world, gen };
}
