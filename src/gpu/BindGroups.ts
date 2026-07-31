import type { Camera } from "../core/Camera";
import type { Config } from "../core/Config";
import type { BindGroupLayouts } from "./BindGroupLayouts";
import type { Resources } from "./Resources";

export type StaticBindGroups = {
  compact: GPUBindGroup;
  clear: GPUBindGroup;
  indirect: GPUBindGroup;
  free: GPUBindGroup;
  gen: GPUBindGroup;
  mip: GPUBindGroup;
  modify: GPUBindGroup;
  store: GPUBindGroup;
  renderStatic: GPUBindGroup;
};

export type DynamicBindGroups = {
  renderDynamic: GPUBindGroup;
  present: GPUBindGroup;
};

export type BindGroups = StaticBindGroups & DynamicBindGroups;

function createFreeGroup(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  resources: Resources,
): GPUBindGroup {
  return device.createBindGroup({
    label: "free bind group",
    layout: layouts.free,
    entries: [
      { binding: 0, resource: { buffer: resources.unload_params } },
      { binding: 1, resource: resources.chunk_index_map.createView() },
      { binding: 2, resource: { buffer: resources.free_list } },
    ],
  });
}

function createClearGroup(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  resources: Resources,
): GPUBindGroup {
  return device.createBindGroup({
    label: "clear bind group",
    layout: layouts.clear,
    entries: [
      { binding: 0, resource: { buffer: resources.unload_params } },
      { binding: 1, resource: resources.chunk_index_map.createView() },
      { binding: 2, resource: { buffer: resources.skip_mip } },
    ],
  });
}

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

function createMipGroup(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  resources: Resources,
): GPUBindGroup {
  return device.createBindGroup({
    label: "mip bind group",
    layout: layouts.mip,
    entries: [
      { binding: 0, resource: { buffer: resources.load_list } },
      { binding: 1, resource: resources.chunk_index_map.createView() },
      { binding: 2, resource: { buffer: resources.skip_mip } },
    ],
  });
}

function createRenderStaticGroup(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  resources: Resources,
): GPUBindGroup {
  return device.createBindGroup({
    label: "render static bind group",
    layout: layouts.renderStatic,
    entries: [
      { binding: 0, resource: { buffer: resources.chunk_pool } },
      { binding: 1, resource: resources.chunk_index_map.createView() },
      { binding: 2, resource: { buffer: resources.gen_flags } },
      { binding: 3, resource: { buffer: resources.skip_mip } },
      { binding: 4, resource: { buffer: resources.block_target } },
    ],
  });
}

function createModifyGroup(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  resources: Resources,
  config: Config,
): GPUBindGroup {
  return device.createBindGroup({
    label: "modify bind group",
    layout: layouts.modify,
    entries: [
      { binding: 0, resource: { buffer: config.buffer } },
      { binding: 1, resource: { buffer: resources.block_target } },
      { binding: 2, resource: resources.chunk_index_map.createView() },
      { binding: 3, resource: { buffer: resources.chunk_pool } },
      { binding: 4, resource: { buffer: resources.free_list } },
      { binding: 5, resource: { buffer: resources.alloc_result } },
    ],
  });
}

function createStoreGroup(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  resources: Resources,
): GPUBindGroup {
  return device.createBindGroup({
    label: "store bind group",
    layout: layouts.store,
    entries: [
      { binding: 0, resource: { buffer: resources.alloc_result } },
      { binding: 1, resource: resources.chunk_index_map.createView() },
      { binding: 2, resource: { buffer: resources.skip_mip } },
    ],
  });
}

export function createStaticBindGroups(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  resources: Resources,
  config: Config,
  camera: Camera,
): StaticBindGroups {
  return {
    free: createFreeGroup(device, layouts, resources),
    clear: createClearGroup(device, layouts, resources),
    compact: createCompactGroup(device, layouts, resources),
    indirect: createIndirectGroup(device, layouts, resources),
    gen: createGenGroup(device, layouts, resources, camera),
    mip: createMipGroup(device, layouts, resources),
    renderStatic: createRenderStaticGroup(device, layouts, resources),
    modify: createModifyGroup(device, layouts, resources, config),
    store: createStoreGroup(device, layouts, resources),
  };
}
