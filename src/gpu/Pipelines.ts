import { link, makeWeslDevice } from "wesl";
import genWesl from "../shaders/gen.wesl?link";
import raytraceWesl from "../shaders/raytrace.wesl?link";
import renderWesl from "../shaders/render.wesl?link";
import { createPipelineLayouts } from "./PipelineLayouts.ts";
import type { BindGroupLayouts } from "./BindGroupLayouts.ts";
import { GEN_SIDE, CHUNK_SIDE_SHIFT, BITS_PER_ID } from "../core/Config.ts";

export type Pipelines = {
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

  const linkedGen = await link({
    ...genWesl,
    constants: { GEN_SIDE, CHUNK_SIDE_SHIFT, BITS_PER_ID },
  });
  const linkedRaytrace = await link({
    ...raytraceWesl,
    constants: { GEN_SIDE, CHUNK_SIDE_SHIFT, BITS_PER_ID },
  });
  const linkedRender = await link(renderWesl);

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
    [genModule, raytraceModule, renderModule].map(validateShader),
  );

  const pipeline_layouts = createPipelineLayouts(device, bind_group_layouts);

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

  return { gen, raytrace, render };
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
