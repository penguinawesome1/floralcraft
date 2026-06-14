import chunkShader from "../shaders/data/Chunk.wgsl?raw";
import worldShader from "../shaders/data/World.wgsl?raw";
import raycastShader from "../shaders/raycast.wgsl?raw";
import renderShader from "../shaders/render.wgsl?raw";
import { createPipelineLayouts } from "./PipelineLayouts.ts";
import type { BindGroupLayouts } from "./BindGroupLayouts.ts";

export type Pipelines = {
  raycast: GPUComputePipeline;
  render: GPURenderPipeline;
};

export async function createPipelines(
  device: GPUDevice,
  format: GPUTextureFormat,
  bind_group_layouts: BindGroupLayouts,
): Promise<Pipelines> {
  const raycastModule = device.createShaderModule({
    label: "raycast shader module",
    code: [chunkShader, worldShader, raycastShader].join("\n"),
  });
  const renderModule = device.createShaderModule({
    label: "render shader module",
    code: renderShader,
  });

  await Promise.all(
    [raycastModule, renderModule].map(validateShader),
  );

  const pipeline_layouts = createPipelineLayouts(device, bind_group_layouts);

  const raycast = device.createComputePipeline({
    label: "raycast pipeline",
    layout: pipeline_layouts.raycast,
    compute: {
      module: raycastModule,
      entryPoint: "cs_main",
      constants: {
        IS_DEBUG_MODE: 0,
        MAX_STEPS: 50,
      },
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

  return { raycast, render };
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
