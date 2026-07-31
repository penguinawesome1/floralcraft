import { link, makeWeslDevice, type LinkParams } from "wesl";
import { createPipelineLayouts } from "./PipelineLayouts.ts";
import type { BindGroupLayouts } from "./BindGroupLayouts.ts";
import { SHADER_CONFIG } from "../core/Config.ts";

import freeWesl from "../shaders/gen/free.wesl?link";
import clearWesl from "../shaders/gen/clear.wesl?link";
import compactWesl from "../shaders/gen/compact.wesl?link";
import indirectWesl from "../shaders/gen/indirect.wesl?link";
import genWesl from "../shaders/gen/gen.wesl?link";
import mipWesl from "../shaders/gen/mip.wesl?link";
import renderWesl from "../shaders/render/render.wesl?link";
import modifyWesl from "../shaders/modify/modify.wesl?link";
import storeWesl from "../shaders/modify/store.wesl?link";
import presentWesl from "../shaders/present.wesl?link";

export type Pipelines = {
  free: GPUComputePipeline;
  clear: GPUComputePipeline;
  compact: GPUComputePipeline;
  indirect: GPUComputePipeline;
  gen: GPUComputePipeline;
  mip: GPUComputePipeline;
  render: GPUComputePipeline;
  modify: GPUComputePipeline;
  store: GPUComputePipeline;
  present: GPURenderPipeline;
};

async function loadShaderModule(
  weslDevice: ReturnType<typeof makeWeslDevice>,
  weslSource: LinkParams,
  label: string,
): Promise<GPUShaderModule> {
  const linked = await link({ ...weslSource, constants: SHADER_CONFIG });
  const module = linked.createShaderModule(weslDevice, { label });
  await validateShader(module);
  return module;
}

async function validateShader(module: GPUShaderModule) {
  const info = await module.getCompilationInfo();
  const errors = info.messages.filter((m) => m.type === "error");
  if (errors.length > 0) {
    for (const e of errors)
      console.error(`WGSL error line ${e.lineNum}: ${e.message}`);
    throw new Error("Shader compilation failed");
  }
}

function createFreePipeline(
  device: GPUDevice,
  layout: GPUPipelineLayout,
  module: GPUShaderModule,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "free pipeline",
    layout,
    compute: { module, entryPoint: "free_slab" },
  });
}

function createClearPipeline(
  device: GPUDevice,
  layout: GPUPipelineLayout,
  module: GPUShaderModule,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "clear pipeline",
    layout,
    compute: { module, entryPoint: "clear_slab" },
  });
}

function createCompactPipeline(
  device: GPUDevice,
  layout: GPUPipelineLayout,
  module: GPUShaderModule,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "compact pipeline",
    layout,
    compute: { module, entryPoint: "compact_load_set" },
  });
}

function createIndirectPipeline(
  device: GPUDevice,
  layout: GPUPipelineLayout,
  module: GPUShaderModule,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "indirect pipeline",
    layout,
    compute: { module, entryPoint: "write_indirect_args" },
  });
}

function createGenPipeline(
  device: GPUDevice,
  layout: GPUPipelineLayout,
  module: GPUShaderModule,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "gen pipeline",
    layout,
    compute: { module, entryPoint: "gen_chunk" },
  });
}

function createMipPipeline(
  device: GPUDevice,
  layout: GPUPipelineLayout,
  module: GPUShaderModule,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "mip pipeline",
    layout,
    compute: { module, entryPoint: "expand_skip_mip" },
  });
}

function createRenderPipeline(
  device: GPUDevice,
  layout: GPUPipelineLayout,
  module: GPUShaderModule,
  is_debug_mode: boolean,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "render pipeline",
    layout,
    compute: {
      module,
      entryPoint: "render",
      constants: { IS_DEBUG_MODE: is_debug_mode ? 1 : 0 },
    },
  });
}

function createModifyPipeline(
  device: GPUDevice,
  layout: GPUPipelineLayout,
  module: GPUShaderModule,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "modify pipeline",
    layout,
    compute: { module, entryPoint: "modify_target_block" },
  });
}

function createStorePipeline(
  device: GPUDevice,
  layout: GPUPipelineLayout,
  module: GPUShaderModule,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "store pipeline",
    layout,
    compute: { module, entryPoint: "store_chunk_num" },
  });
}

function createPresentPipeline(
  device: GPUDevice,
  layout: GPUPipelineLayout,
  module: GPUShaderModule,
  format: GPUTextureFormat,
): GPURenderPipeline {
  return device.createRenderPipeline({
    label: "present pipeline",
    layout,
    vertex: { module, entryPoint: "vs_main" },
    fragment: {
      module,
      entryPoint: "fs_main",
      targets: [{ format }],
    },
    primitive: { topology: "triangle-list" },
  });
}

export async function createPipelines(
  device: GPUDevice,
  format: GPUTextureFormat,
  bind_group_layouts: BindGroupLayouts,
  is_debug_mode: boolean,
): Promise<Pipelines> {
  const weslDevice = makeWeslDevice(device);

  const freeModule = await loadShaderModule(
    weslDevice,
    freeWesl,
    "free shader module",
  );
  const clearModule = await loadShaderModule(
    weslDevice,
    clearWesl,
    "clear shader module",
  );
  const compactModule = await loadShaderModule(
    weslDevice,
    compactWesl,
    "compact shader module",
  );
  const indirectModule = await loadShaderModule(
    weslDevice,
    indirectWesl,
    "indirect shader module",
  );
  const genModule = await loadShaderModule(
    weslDevice,
    genWesl,
    "gen shader module",
  );
  const mipModule = await loadShaderModule(
    weslDevice,
    mipWesl,
    "mip shader module",
  );
  const renderModule = await loadShaderModule(
    weslDevice,
    renderWesl,
    "render shader module",
  );
  const modifyModule = await loadShaderModule(
    weslDevice,
    modifyWesl,
    "modify shader module",
  );
  const storeModule = await loadShaderModule(
    weslDevice,
    storeWesl,
    "store shader module",
  );
  const presentModule = await loadShaderModule(
    weslDevice,
    presentWesl,
    "present shader module",
  );

  const pipeline_layouts = createPipelineLayouts(device, bind_group_layouts);

  return {
    free: createFreePipeline(device, pipeline_layouts.free, freeModule),
    clear: createClearPipeline(device, pipeline_layouts.clear, clearModule),
    compact: createCompactPipeline(
      device,
      pipeline_layouts.compact,
      compactModule,
    ),
    indirect: createIndirectPipeline(
      device,
      pipeline_layouts.indirect,
      indirectModule,
    ),
    gen: createGenPipeline(device, pipeline_layouts.gen, genModule),
    mip: createMipPipeline(device, pipeline_layouts.mip, mipModule),
    render: createRenderPipeline(
      device,
      pipeline_layouts.render,
      renderModule,
      is_debug_mode,
    ),
    modify: createModifyPipeline(device, pipeline_layouts.modify, modifyModule),
    store: createStorePipeline(device, pipeline_layouts.store, storeModule),
    present: createPresentPipeline(
      device,
      pipeline_layouts.present,
      presentModule,
      format,
    ),
  };
}
