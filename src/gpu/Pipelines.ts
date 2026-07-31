import { link, makeWeslDevice, type LinkParams } from "wesl";
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
  bgLayout: GPUBindGroupLayout,
  module: GPUShaderModule,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "free pipeline",
    layout: device.createPipelineLayout({
      label: "free pipeline layout",
      bindGroupLayouts: [bgLayout],
    }),
    compute: { module, entryPoint: "free_slab" },
  });
}

function createClearPipeline(
  device: GPUDevice,
  bgLayout: GPUBindGroupLayout,
  module: GPUShaderModule,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "clear pipeline",
    layout: device.createPipelineLayout({
      label: "clear pipeline layout",
      bindGroupLayouts: [bgLayout],
    }),
    compute: { module, entryPoint: "clear_slab" },
  });
}

function createCompactPipeline(
  device: GPUDevice,
  bgLayout: GPUBindGroupLayout,
  module: GPUShaderModule,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "compact pipeline",
    layout: device.createPipelineLayout({
      label: "compact pipeline layout",
      bindGroupLayouts: [bgLayout],
    }),
    compute: { module, entryPoint: "compact_load_set" },
  });
}

function createIndirectPipeline(
  device: GPUDevice,
  bgLayout: GPUBindGroupLayout,
  module: GPUShaderModule,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "indirect pipeline",
    layout: device.createPipelineLayout({
      label: "indirect pipeline layout",
      bindGroupLayouts: [bgLayout],
    }),
    compute: { module, entryPoint: "write_indirect_args" },
  });
}

function createGenPipeline(
  device: GPUDevice,
  bgLayout: GPUBindGroupLayout,
  module: GPUShaderModule,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "gen pipeline",
    layout: device.createPipelineLayout({
      label: "gen pipeline layout",
      bindGroupLayouts: [bgLayout],
    }),
    compute: { module, entryPoint: "gen_chunk" },
  });
}

function createMipPipeline(
  device: GPUDevice,
  bgLayout: GPUBindGroupLayout,
  module: GPUShaderModule,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "mip pipeline",
    layout: device.createPipelineLayout({
      label: "mip pipeline layout",
      bindGroupLayouts: [bgLayout],
    }),
    compute: { module, entryPoint: "expand_skip_mip" },
  });
}

function createRenderPipeline(
  device: GPUDevice,
  bgLayouts: GPUBindGroupLayout[],
  module: GPUShaderModule,
  isDebugMode: boolean,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "render pipeline",
    layout: device.createPipelineLayout({
      label: "render pipeline layout",
      bindGroupLayouts: bgLayouts,
    }),
    compute: {
      module,
      entryPoint: "render",
      constants: { IS_DEBUG_MODE: isDebugMode ? 1 : 0 },
    },
  });
}

function createModifyPipeline(
  device: GPUDevice,
  bgLayout: GPUBindGroupLayout,
  module: GPUShaderModule,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "modify pipeline",
    layout: device.createPipelineLayout({
      label: "modify pipeline layout",
      bindGroupLayouts: [bgLayout],
    }),
    compute: { module, entryPoint: "modify_target_block" },
  });
}

function createStorePipeline(
  device: GPUDevice,
  bgLayout: GPUBindGroupLayout,
  module: GPUShaderModule,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "store pipeline",
    layout: device.createPipelineLayout({
      label: "store pipeline layout",
      bindGroupLayouts: [bgLayout],
    }),
    compute: { module, entryPoint: "store_chunk_num" },
  });
}

function createPresentPipeline(
  device: GPUDevice,
  bgLayout: GPUBindGroupLayout,
  module: GPUShaderModule,
  format: GPUTextureFormat,
): GPURenderPipeline {
  return device.createRenderPipeline({
    label: "present pipeline",
    layout: device.createPipelineLayout({
      label: "present pipeline layout",
      bindGroupLayouts: [bgLayout],
    }),
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
  bgLayouts: BindGroupLayouts,
  isDebugMode: boolean,
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

  return {
    free: createFreePipeline(device, bgLayouts.free, freeModule),
    clear: createClearPipeline(device, bgLayouts.clear, clearModule),
    compact: createCompactPipeline(device, bgLayouts.compact, compactModule),
    indirect: createIndirectPipeline(
      device,
      bgLayouts.indirect,
      indirectModule,
    ),
    gen: createGenPipeline(device, bgLayouts.gen, genModule),
    mip: createMipPipeline(device, bgLayouts.mip, mipModule),
    render: createRenderPipeline(
      device,
      [bgLayouts.renderStatic, bgLayouts.renderDynamic],
      renderModule,
      isDebugMode,
    ),
    modify: createModifyPipeline(device, bgLayouts.modify, modifyModule),
    store: createStorePipeline(device, bgLayouts.store, storeModule),
    present: createPresentPipeline(
      device,
      bgLayouts.present,
      presentModule,
      format,
    ),
  };
}
