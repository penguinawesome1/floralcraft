import type { Config } from "../core/Config";
import type { BindGroupLayouts } from "./BindGroupLayouts";
import type { Resources } from "./Resources";

export type StaticBindGroups = {
  movement: GPUBindGroup;
  prepUnload: GPUBindGroup;
  free: GPUBindGroup;
  clear: GPUBindGroup;
  prepGen: GPUBindGroup;
  compact: GPUBindGroup;
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

function createMovementGroup(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  resources: Resources,
  config: Config,
): GPUBindGroup {
  return device.createBindGroup({
    label: "movement bind group",
    layout: layouts.movement,
    entries: [
      { binding: 0, resource: { buffer: config.buffer } },
      { binding: 1, resource: { buffer: resources.player } },
      { binding: 2, resource: { buffer: resources.chunk_pool } },
      { binding: 3, resource: resources.chunk_index_map.createView() },
    ],
  });
}

function createPrepUnloadGroup(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  resources: Resources,
): GPUBindGroup {
  return device.createBindGroup({
    label: "prep unload bind group",
    layout: layouts.prepUnload,
    entries: [
      { binding: 0, resource: { buffer: resources.player } },
      { binding: 1, resource: { buffer: resources.indirect_args } },
      { binding: 2, resource: { buffer: resources.unload_params } },
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

function createPrepGenGroup(
  device: GPUDevice,
  layouts: BindGroupLayouts,
  resources: Resources,
): GPUBindGroup {
  return device.createBindGroup({
    label: "prep gen bind group",
    layout: layouts.prepGen,
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
): GPUBindGroup {
  return device.createBindGroup({
    label: "gen bind group",
    layout: layouts.gen,
    entries: [
      { binding: 0, resource: { buffer: resources.load_list } },
      { binding: 1, resource: { buffer: resources.player } },
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
  config: Config,
): GPUBindGroup {
  return device.createBindGroup({
    label: "render static bind group",
    layout: layouts.renderStatic,
    entries: [
      { binding: 0, resource: { buffer: config.buffer } },
      { binding: 1, resource: { buffer: resources.player } },
      { binding: 2, resource: { buffer: resources.chunk_pool } },
      { binding: 3, resource: resources.chunk_index_map.createView() },
      { binding: 4, resource: { buffer: resources.gen_flags } },
      { binding: 5, resource: { buffer: resources.skip_mip } },
      { binding: 6, resource: { buffer: resources.block_target } },
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
      { binding: 6, resource: { buffer: resources.player } },
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
): StaticBindGroups {
  return {
    movement: createMovementGroup(device, layouts, resources, config),
    prepUnload: createPrepUnloadGroup(device, layouts, resources),
    free: createFreeGroup(device, layouts, resources),
    clear: createClearGroup(device, layouts, resources),
    compact: createCompactGroup(device, layouts, resources),
    prepGen: createPrepGenGroup(device, layouts, resources),
    gen: createGenGroup(device, layouts, resources),
    mip: createMipGroup(device, layouts, resources),
    renderStatic: createRenderStaticGroup(device, layouts, resources, config),
    modify: createModifyGroup(device, layouts, resources, config),
    store: createStoreGroup(device, layouts, resources),
  };
}
