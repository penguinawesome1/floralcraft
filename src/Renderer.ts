import raycastShader from "./shaders/raycast.wgsl?raw";
import renderShader from "./shaders/render.wgsl?raw";
import { Camera } from "./Camera";
import { type InputState } from "./Input.ts";
import { vec3 } from "gl-matrix";

export class Renderer {
  private canvas: HTMLCanvasElement;
  private device!: GPUDevice;
  private context!: GPUCanvasContext;
  private format!: GPUTextureFormat;
  private renderTarget!: GPUTexture;
  private computePipeline!: GPUComputePipeline;
  private renderPipeline!: GPURenderPipeline;
  private canvasSampler!: GPUSampler;
  private renderBindGroup!: GPUBindGroup;
  private computeBindGroup!: GPUBindGroup;
  private camera!: Camera;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
  }

  async init() {
    if (!navigator.gpu) throw new Error("WebGPU not supported");

    const adapter = await navigator.gpu.requestAdapter();
    if (!adapter) throw new Error("No GPU adapter found");
    this.device = await adapter.requestDevice();

    this.context = this.canvas.getContext("webgpu")!;
    this.format = navigator.gpu.getPreferredCanvasFormat();
    this.camera = new Camera(this.device, 0.002, 0.1, vec3.fromValues(2, 3, 7));

    this.createPipelines();
    this.resize();

    window.addEventListener("resize", () => this.resize());
  }

  update(input_state: InputState) {
    this.camera.update(this.device.queue, input_state);
  }

  frame() {
    const commandEncoder = this.device.createCommandEncoder();

    const computePass = commandEncoder.beginComputePass();
    computePass.setPipeline(this.computePipeline);
    computePass.setBindGroup(0, this.computeBindGroup);

    const workgroupCountX = Math.ceil(this.canvas.width / 8);
    const workgroupCountY = Math.ceil(this.canvas.height / 8);
    computePass.dispatchWorkgroups(workgroupCountX, workgroupCountY);

    computePass.end();

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

    renderPass.setPipeline(this.renderPipeline);
    renderPass.setBindGroup(0, this.renderBindGroup);

    renderPass.draw(3);

    renderPass.end();

    const commandBuffer = commandEncoder.finish();
    this.device.queue.submit([commandBuffer]);
  }

  private resize() {
    const dpr = window.devicePixelRatio;
    const w = this.canvas.clientWidth * dpr;
    const h = this.canvas.clientHeight * dpr;

    if (w == this.canvas.width && h == this.canvas.height && this.renderTarget)
      return;

    this.canvas.width = w;
    this.canvas.height = h;

    this.context.configure({
      device: this.device,
      format: this.format,
      alphaMode: "opaque",
    });

    this.renderTarget = this.device.createTexture({
      size: [this.canvas.width, this.canvas.height],
      format: "rgba8unorm",
      usage: GPUTextureUsage.STORAGE_BINDING | GPUTextureUsage.TEXTURE_BINDING,
    });

    this.configureViewsAndBindGroups();
  }

  private createPipelines() {
    const raycastShaderModule = this.device.createShaderModule({
      code: raycastShader,
    });
    const renderShaderModule = this.device.createShaderModule({
      code: renderShader,
    });

    this.computePipeline = this.device.createComputePipeline({
      layout: "auto",
      compute: { module: raycastShaderModule, entryPoint: "cs_main" },
    });

    this.renderPipeline = this.device.createRenderPipeline({
      layout: "auto",
      vertex: { module: renderShaderModule, entryPoint: "vs_main" },
      fragment: {
        module: renderShaderModule,
        entryPoint: "fs_main",
        targets: [{ format: this.format }],
      },
      primitive: { topology: "triangle-list" },
    });

    this.canvasSampler = this.device.createSampler({
      magFilter: "nearest",
      minFilter: "nearest",
      mipmapFilter: "nearest",
    });
  }

  private configureViewsAndBindGroups() {
    const renderTargetView = this.renderTarget.createView();

    this.computeBindGroup = this.device.createBindGroup({
      layout: this.computePipeline.getBindGroupLayout(0),
      entries: [
        { binding: 0, resource: renderTargetView },
        { binding: 1, resource: this.camera.buffer },
      ],
    });

    this.renderBindGroup = this.device.createBindGroup({
      layout: this.renderPipeline.getBindGroupLayout(0),
      entries: [
        { binding: 0, resource: renderTargetView },
        { binding: 1, resource: this.canvasSampler },
      ],
    });
  }
}
