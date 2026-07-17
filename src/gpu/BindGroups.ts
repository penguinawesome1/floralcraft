import type { Camera } from "../core/Camera";
import type { BindGroupLayouts } from "./BindGroupLayouts";
import type { Resources } from "./Resources";

export type StaticBindGroups = {
  compact: GPUBindGroup;
  indirect: GPUBindGroup;
  free: GPUBindGroup;
  gen: GPUBindGroup;
  raytraceStatic: GPUBindGroup;
};

export type DynamicBindGroups = {
  raytraceDynamic: GPUBindGroup;
  render: GPUBindGroup;
};

export type BindGroups = StaticBindGroups & DynamicBindGroups;

function createCompactGroup(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  resources: Resources,
): GPUBindGroup {
  return device.createBindGroup({
    label: "compact bind group",
    layout: layouts.compact,
    entries: [
      { binding: 0, resource: { buffer: resources.gen_flags } },
      { binding: 1, resource: { buffer: resources.load_list } },
    ],
  });
}

function createIndirectGroup(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  resources: Resources,
): GPUBindGroup {
  return device.createBindGroup({
    label: "indirect bind group",
    layout: layouts.indirect,
    entries: [
      { binding: 0, resource: { buffer: resources.load_list } },
      { binding: 1, resource: { buffer: resources.indirect_args } },
    ],
  });
}

function createFreeGroup(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  resources: Resources,
): GPUBindGroup {
  return device.createBindGroup({
    label: "free bind group",
    layout: layouts.free,
    entries: [
      { binding: 0, resource: { buffer: resources.load_list } },
      { binding: 1, resource: resources.chunk_index_map.createView() },
      { binding: 2, resource: { buffer: resources.free_list } },
    ],
  });
}

function createGenGroup(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  resources: Resources,
  camera: Camera,
): GPUBindGroup {
  return device.createBindGroup({
    label: "gen bind group",
    layout: layouts.gen,
    entries: [
      { binding: 0, resource: { buffer: resources.load_list } },
      { binding: 1, resource: { buffer: camera.buffer } },
      { binding: 2, resource: { buffer: resources.free_list } },
      { binding: 3, resource: resources.chunk_index_map.createView() },
      { binding: 4, resource: { buffer: resources.chunk_pool } },
    ],
  });
}

function createRaytraceStaticGroup(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  resources: Resources,
): GPUBindGroup {
  return device.createBindGroup({
    label: "raytrace static bind group",
    layout: layouts.raytraceStatic,
    entries: [
      { binding: 0, resource: { buffer: resources.chunk_pool } },
      { binding: 1, resource: resources.chunk_index_map.createView() },
      { binding: 2, resource: { buffer: resources.gen_flags } },
      { binding: 3, resource: { buffer: resources.skip_mips } },
    ],
  });
}

export function createStaticBindGroups(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  resources: Resources,
  camera: Camera,
): StaticBindGroups {
  return {
    compact: createCompactGroup(device, layouts, resources),
    indirect: createIndirectGroup(device, layouts, resources),
    free: createFreeGroup(device, layouts, resources),
    gen: createGenGroup(device, layouts, resources, camera),
    raytraceStatic: createRaytraceStaticGroup(device, layouts, resources),
  };
}
