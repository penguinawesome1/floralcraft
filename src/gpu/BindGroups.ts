import type { BindGroupLayouts } from "./BindGroupLayouts";
import type { Resources } from "./Resources";

export type StaticBindGroups = {
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
): StaticBindGroups {
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
    ],
  });

  return { gen, raytraceStatic };
}
