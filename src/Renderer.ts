import { Camera } from "./Camera";
import { type InputState } from "./Input.ts";
import { vec3 } from "gl-matrix";
import {
  type BindGroupLayouts,
  createBindGroupLayouts,
} from "./render/BindGroupLayouts.ts";
import {
  type BindGroups,
  type DynamicBindGroups,
  createStaticBindGroups,
} from "./render/BindGroups.ts";
import { type Buffers, createBuffers } from "./render/Buffers.ts";
import { type Pipelines, createPipelines } from "./render/Pipelines.ts";

export class Renderer {
  private readonly canvas: HTMLCanvasElement;
  private device!: GPUDevice;
  private context!: GPUCanvasContext;
  private format!: GPUTextureFormat;
  private canvasSampler!: GPUSampler;
  private renderTarget!: GPUTexture;
  private camera!: Camera;

  private buffers!: Buffers;
  private bindGroupLayouts!: BindGroupLayouts;
  private bindGroups!: BindGroups;
  private pipelines!: Pipelines;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
  }

  async init(): Promise<void> {
    if (!navigator.gpu) throw new Error("WebGPU not supported");

    const adapter = await navigator.gpu.requestAdapter();
    if (!adapter) throw new Error("No GPU adapter found");
    this.device = await adapter.requestDevice();

    this.context = this.canvas.getContext("webgpu")!;
    this.format = navigator.gpu.getPreferredCanvasFormat();
    this.canvasSampler = this.device.createSampler({
      magFilter: "nearest",
      minFilter: "nearest",
      mipmapFilter: "nearest",
    });
    this.camera = new Camera(this.device, 0.002, 0.2, vec3.fromValues(2, 3, 7));

    this.bindGroupLayouts = createBindGroupLayouts(this.device);
    this.buffers = createBuffers(this.device);
    const staticBindGroups = createStaticBindGroups(
      this.device,
      this.bindGroupLayouts,
      this.buffers,
    );
    this.bindGroups = { ...staticBindGroups } as BindGroups;
    this.pipelines = await createPipelines(
      this.device,
      this.format,
      this.bindGroupLayouts,
    );

    const observer = new ResizeObserver(() => this.resize());
    observer.observe(this.canvas);
    this.resize();
  }

  update(inputState: InputState): void {
    this.camera.update(this.device.queue, inputState);
  }

  frame(): void {
    const commandEncoder = this.device.createCommandEncoder();
    this.encodeGenPass(commandEncoder);
    this.encodeRaycastPass(commandEncoder);
    this.encodeRenderPass(commandEncoder);
    this.device.queue.submit([commandEncoder.finish()]);
  }

  private encodeGenPass(commandEncoder: GPUCommandEncoder): void {
    const pass = commandEncoder.beginComputePass();
    pass.setPipeline(this.pipelines.gen);
    pass.setBindGroup(0, this.bindGroups.atomic_world);
    pass.dispatchWorkgroups(30, 1, 30);
    pass.end();
  }

  private encodeRaycastPass(commandEncoder: GPUCommandEncoder): void {
    const pass = commandEncoder.beginComputePass();
    pass.setPipeline(this.pipelines.raycast);
    pass.setBindGroup(0, this.bindGroups.world);
    pass.setBindGroup(1, this.bindGroups.raycast);
    pass.dispatchWorkgroups(
      Math.ceil(this.canvas.width),
      Math.ceil(this.canvas.height),
    );
    pass.end();
  }

  private encodeRenderPass(commandEncoder: GPUCommandEncoder): void {
    const canvasTextureView = this.context.getCurrentTexture().createView();
    const renderPass = commandEncoder.beginRenderPass({
      colorAttachments: [
        {
          view: canvasTextureView,
          clearValue: [0, 0, 0, 1],
          loadOp: "clear",
          storeOp: "store",
        },
      ],
    });
    renderPass.setPipeline(this.pipelines.render);
    renderPass.setBindGroup(0, this.bindGroups.render);
    renderPass.draw(3);
    renderPass.end();
  }

  private resize(): void {
    const dpr = window.devicePixelRatio;
    const w = this.canvas.clientWidth * dpr;
    const h = this.canvas.clientHeight * dpr;

    if (
      w === this.canvas.width &&
      h === this.canvas.height &&
      this.renderTarget
    )
      return;

    this.canvas.width = w;
    this.canvas.height = h;

    this.context.configure({
      device: this.device,
      format: this.format,
      alphaMode: "opaque",
    });

    if (this.renderTarget) {
      this.renderTarget.destroy();
    }

    this.renderTarget = this.device.createTexture({
      size: [this.canvas.width, this.canvas.height],
      format: "rgba8unorm",
      usage: GPUTextureUsage.STORAGE_BINDING | GPUTextureUsage.TEXTURE_BINDING,
    });

    this.bindGroups = { ...this.bindGroups, ...this.createDynamicBindGroups() };
  }

  private createDynamicBindGroups(): DynamicBindGroups {
    const renderTargetView = this.renderTarget.createView();

    const raycast = this.device.createBindGroup({
      label: "raycast bind group",
      layout: this.bindGroupLayouts.raycast,
      entries: [
        { binding: 0, resource: renderTargetView },
        { binding: 1, resource: { buffer: this.camera.buffer } },
      ],
    });

    const render = this.device.createBindGroup({
      label: "render bind group",
      layout: this.bindGroupLayouts.render,
      entries: [
        { binding: 0, resource: renderTargetView },
        { binding: 1, resource: this.canvasSampler },
      ],
    });

    return { raycast, render };
  }
}
