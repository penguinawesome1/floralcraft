import genShader from "../shaders/gen.wgsl";
import raycastShader from "../shaders/raycast/main.wgsl";
import renderShader from "../shaders/render.wgsl";
import { createPipelineLayouts } from "./PipelineLayouts.ts";
import type { BindGroupLayouts } from "./BindGroupLayouts.ts";

export type Pipelines = {
  gen: GPUComputePipeline;
  raycast: GPUComputePipeline;
  render: GPURenderPipeline;
};

export async function createPipelines(
  device: GPUDevice,
  format: GPUTextureFormat,
  bind_group_layouts: BindGroupLayouts,
  is_debug_mode: boolean,
): Promise<Pipelines> {
  const genModule = device.createShaderModule({
    label: "gen shader module",
    code: genShader,
  });
  const raycastModule = device.createShaderModule({
    label: "raycast shader module",
    code: raycastShader,
  });
  const renderModule = device.createShaderModule({
    label: "render shader module",
    code: renderShader,
  });

  await Promise.all(
    [genModule, raycastModule, renderModule].map(validateShader),
  );

  const pipeline_layouts = createPipelineLayouts(device, bind_group_layouts);

  const gen = device.createComputePipeline({
    label: "gen pipeline",
    layout: pipeline_layouts.gen,
    compute: {
      module: genModule,
      entryPoint: "cs_main",
    },
  });
  const raycast = device.createComputePipeline({
    label: "raycast pipeline",
    layout: pipeline_layouts.raycast,
    compute: {
      module: raycastModule,
      entryPoint: "cs_main",
      constants: { config__IS_DEBUG_MODE: is_debug_mode ? 1 : 0 },
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

  return { gen, raycast, render };
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
