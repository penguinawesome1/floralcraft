import { link, makeWeslDevice, type LinkParams } from "wesl";
import compactWesl from "../shaders/compact.wesl?link";
import indirectWesl from "../shaders/indirect.wesl?link";
import genWesl from "../shaders/gen.wesl?link";
import raytraceWesl from "../shaders/raytrace.wesl?link";
import renderWesl from "../shaders/render.wesl?link";
import { createPipelineLayouts } from "./PipelineLayouts.ts";
import type { BindGroupLayouts } from "./BindGroupLayouts.ts";
import { SHADER_CONFIG } from "../core/Config.ts";

export type Pipelines = {
  compact: GPUComputePipeline;
  indirect: GPUComputePipeline;
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
      entryPoint: "cs_main",
      constants: { IS_DEBUG_MODE: is_debug_mode ? 1 : 0 },
    },
  });
}

function createRenderPipeline(
  device: GPUDevice,
  layout: GPUPipelineLayout,
  module: GPUShaderModule,
  format: GPUTextureFormat,
): GPURenderPipeline {
  return device.createRenderPipeline({
    label: "render pipeline",
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
  const renderModule = await loadShaderModule(
    weslDevice,
    renderWesl,
    "render shader module",
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
    gen: createGenPipeline(device, pipeline_layouts.gen, genModule),
    raytrace: createRaytracePipeline(
      device,
      pipeline_layouts.raytrace,
      raytraceModule,
      is_debug_mode,
    ),
    render: createRenderPipeline(
      device,
      pipeline_layouts.render,
      renderModule,
      format,
    ),
  };
}
