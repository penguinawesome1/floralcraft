import { link, makeWeslDevice } from "wesl";
import compactWesl from "../shaders/compact.wesl?link";
import indirectWesl from "../shaders/indirect.wesl?link";
import genWesl from "../shaders/gen.wesl?link";
import raytraceWesl from "../shaders/raytrace.wesl?link";
import renderWesl from "../shaders/render.wesl?link";
import { createPipelineLayouts } from "./PipelineLayouts.ts";
import type { BindGroupLayouts } from "./BindGroupLayouts.ts";
import {
  GEN_SIDE,
  CHUNK_SIDE_SHIFT,
  BITS_PER_ID,
  MAX_CHUNK_BATCH_SIZE,
  MAX_CHUNKS_LOADED,
} from "../core/Config.ts";

export type Pipelines = {
  compact: GPUComputePipeline;
  indirect: GPUComputePipeline;
  gen: GPUComputePipeline;
  raytrace: GPUComputePipeline;
  render: GPURenderPipeline;
};

export async function createPipelines(
  device: GPUDevice,
  format: GPUTextureFormat,
  bind_group_layouts: BindGroupLayouts,
  is_debug_mode: boolean,
): Promise<Pipelines> {
  const weslDevice = makeWeslDevice(device);

  const SHADER_CONFIG = {
    GEN_SIDE,
    MAX_CHUNK_BATCH_SIZE,
    CHUNK_SIDE_SHIFT,
    BITS_PER_ID,
    MAX_CHUNKS_LOADED,
  };

  const compactLoad = await link({
    ...compactWesl,
    constants: SHADER_CONFIG,
  });
  const indirectLoad = await link({
    ...indirectWesl,
    constants: SHADER_CONFIG,
  });
  const linkedGen = await link({
    ...genWesl,
    constants: SHADER_CONFIG,
  });
  const linkedRaytrace = await link({
    ...raytraceWesl,
    constants: SHADER_CONFIG,
  });
  const linkedRender = await link(renderWesl);

  const compactModule = compactLoad.createShaderModule(weslDevice, {
    label: "compact shader module",
  });
  const indirectModule = indirectLoad.createShaderModule(weslDevice, {
    label: "indirect shader module",
  });
  const genModule = linkedGen.createShaderModule(weslDevice, {
    label: "gen shader module",
  });
  const raytraceModule = linkedRaytrace.createShaderModule(weslDevice, {
    label: "raytrace shader module",
  });
  const renderModule = linkedRender.createShaderModule(weslDevice, {
    label: "render shader module",
  });

  await Promise.all(
    [
      compactModule,
      indirectModule,
      genModule,
      raytraceModule,
      renderModule,
    ].map(validateShader),
  );

  const pipeline_layouts = createPipelineLayouts(device, bind_group_layouts);

  const compact = device.createComputePipeline({
    label: "compact pipeline",
    layout: pipeline_layouts.compact,
    compute: {
      module: compactModule,
      entryPoint: "compact_load_set",
    },
  });

  const indirect = device.createComputePipeline({
    label: "indirect pipeline",
    layout: pipeline_layouts.indirect,
    compute: {
      module: indirectModule,
      entryPoint: "write_indirect_args",
    },
  });

  const gen = device.createComputePipeline({
    label: "gen pipeline",
    layout: pipeline_layouts.gen,
    compute: {
      module: genModule,
      entryPoint: "gen_chunk",
    },
  });

  const raytrace = device.createComputePipeline({
    label: "raytrace pipeline",
    layout: pipeline_layouts.raytrace,
    compute: {
      module: raytraceModule,
      entryPoint: "cs_main",
      constants: { IS_DEBUG_MODE: is_debug_mode ? 1 : 0 },
    },
  });

  const render = device.createRenderPipeline({
    label: "render pipeline",
    layout: pipeline_layouts.render,
    vertex: { module: renderModule, entryPoint: "vs_main" },
    fragment: {
      module: renderModule,
      entryPoint: "fs_main",
      targets: [{ format }],
    },
    primitive: { topology: "triangle-list" },
  });

  return { compact, indirect, gen, raytrace, render };
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
