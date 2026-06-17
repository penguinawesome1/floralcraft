import readChunkShader from "../shaders/data/read/Chunk.wgsl?raw";
import readWriteChunkShader from "../shaders/data/read-write/Chunk.wgsl?raw";
import readWorldShader from "../shaders/data/read/World.wgsl?raw";
import readWriteWorldShader from "../shaders/data/read-write/World.wgsl?raw";
import genShader from "../shaders/gen.wgsl?raw";
import raycastShader from "../shaders/raycast.wgsl?raw";
import renderShader from "../shaders/render.wgsl?raw";
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
): Promise<Pipelines> {
  const SVO_DEPTH = 8;
  const SVO_BRANCHES_CAPACITY = (Math.pow(8, SVO_DEPTH) - 1) / 7;
  const svoConsts = `
    const SVO_DEPTH = ${SVO_DEPTH}u;
    const SVO_BRANCHES_CAPACITY = ${SVO_BRANCHES_CAPACITY}u;
  `;

  const genModule = device.createShaderModule({
    label: "gen shader module",
    code: [svoConsts, readWriteChunkShader, readWriteWorldShader, genShader].join("\n"),
  });
  const raycastModule = device.createShaderModule({
    label: "raycast shader module",
    code: [svoConsts, readChunkShader, readWorldShader, raycastShader].join("\n"),
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
      constants: {
        IS_DEBUG_MODE: 0,
        MAX_DIST: 100,
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
