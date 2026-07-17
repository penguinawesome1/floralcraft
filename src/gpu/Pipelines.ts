import { link, makeWeslDevice, type LinkParams } from "wesl";
import { createPipelineLayouts } from "./PipelineLayouts.ts";
import type { BindGroupLayouts } from "./BindGroupLayouts.ts";
import { SHADER_CONFIG } from "../core/Config.ts";

import compactWesl from "../shaders/gen/01_compact.wesl?link";
import indirectWesl from "../shaders/gen/02_indirect.wesl?link";
import freeWesl from "../shaders/gen/03_free.wesl?link";
import genWesl from "../shaders/gen/04_gen.wesl?link";
import raytraceWesl from "../shaders/raytrace/05_renderer.wesl?link";
import presentWesl from "../shaders/06_present.wesl?link";

export type Pipelines = {
  compact: GPUComputePipeline;
  indirect: GPUComputePipeline;
  free: GPUComputePipeline;
  gen: GPUComputePipeline;
  raytrace: GPUComputePipeline;
  render: GPURenderPipeline;
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

function createFreePipeline(
  device: GPUDevice,
  layout: GPUPipelineLayout,
  module: GPUShaderModule,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "free pipeline",
    layout,
    compute: { module, entryPoint: "free_chunk" },
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

function createRaytracePipeline(
  device: GPUDevice,
  layout: GPUPipelineLayout,
  module: GPUShaderModule,
  is_debug_mode: boolean,
): GPUComputePipeline {
  return device.createComputePipeline({
    label: "raytrace pipeline",
    layout,
    compute: {
      module,
      entryPoint: "render",
      constants: { IS_DEBUG_MODE: is_debug_mode ? 1 : 0 },
    },
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
  const freeModule = await loadShaderModule(
    weslDevice,
    freeWesl,
    "free shader module",
  );
  const genModule = await loadShaderModule(
    weslDevice,
    genWesl,
    "gen shader module",
  );
  const raytraceModule = await loadShaderModule(
    weslDevice,
    raytraceWesl,
    "raytrace shader module",
  );
  const presentModule = await loadShaderModule(
    weslDevice,
    presentWesl,
    "present shader module",
  );

  const pipeline_layouts = createPipelineLayouts(device, bind_group_layouts);

  return {
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
    free: createFreePipeline(device, pipeline_layouts.free, freeModule),
    gen: createGenPipeline(device, pipeline_layouts.gen, genModule),
    raytrace: createRaytracePipeline(
      device,
      pipeline_layouts.raytrace,
      raytraceModule,
      is_debug_mode,
    ),
    render: createPresentPipeline(
      device,
      pipeline_layouts.render,
      presentModule,
      format,
    ),
  };
}
