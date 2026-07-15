import type { Camera } from "../core/Camera";
import type { BindGroupLayouts } from "./BindGroupLayouts";
import type { Resources } from "./Resources";

export type StaticBindGroups = {
  compact: GPUBindGroup;
  indirect: GPUBindGroup;
  gen: GPUBindGroup;
  raytraceStatic: GPUBindGroup;
};

export type DynamicBindGroups = {
  raytraceDynamic: GPUBindGroup;
  render: GPUBindGroup;
};

export type BindGroups = StaticBindGroups & DynamicBindGroups;

export function createStaticBindGroups(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  resources: Resources,
  camera: Camera,
): StaticBindGroups {
  const compact = device.createBindGroup({
    label: "compact bind group",
    layout: layouts.compact,
    entries: [
      {
        binding: 0,
        resource: { buffer: resources.gen_flags },
      },
      {
        binding: 1,
        resource: { buffer: resources.load_list },
      },
    ],
  });

  const indirect = device.createBindGroup({
    label: "indirect bind group",
    layout: layouts.indirect,
    entries: [
      {
        binding: 0,
        resource: { buffer: resources.indirect_args },
      },
      {
        binding: 1,
        resource: { buffer: resources.load_list },
      },
    ],
  });

  const gen = device.createBindGroup({
    label: "gen bind group",
    layout: layouts.gen,
    entries: [
      {
        binding: 0,
        resource: { buffer: resources.chunk_pool },
      },
      {
        binding: 1,
        resource: resources.chunk_index_map.createView(),
      },
      { binding: 2, resource: { buffer: resources.free_list } },
      { binding: 3, resource: { buffer: camera.buffer } },
      { binding: 4, resource: { buffer: resources.load_list } },
    ],
  });

  const raytraceStatic = device.createBindGroup({
    label: "raytrace static bind group",
    layout: layouts.raytraceStatic,
    entries: [
      {
        binding: 0,
        resource: { buffer: resources.chunk_pool },
      },
      {
        binding: 1,
        resource: resources.chunk_index_map.createView(),
      },
      { binding: 2, resource: resources.gen_flags },
    ],
  });

  return { compact, indirect, gen, raytraceStatic };
}
