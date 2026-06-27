import { Camera } from "./Camera";
import { type InputState } from "./Input.ts";
import { vec3 } from "gl-matrix";
import {
  type BindGroupLayouts,
  createBindGroupLayouts,
} from "../gpu/BindGroupLayouts.ts";
import {
  type BindGroups,
  type DynamicBindGroups,
  createStaticBindGroups,
} from "../gpu/BindGroups.ts";
import { type Buffers, createBuffers } from "../gpu/Buffers.ts";
import { type Pipelines, createPipelines } from "../gpu/Pipelines.ts";
import { type Config, createConfig } from "./Config.ts";

const RING_SIZE = 10;
const RESIZE_DEBOUNCE_MS = 100;

export class Renderer {
  private readonly canvas: HTMLCanvasElement;
  private resizeDebounce: ReturnType<typeof setTimeout> | undefined;
  private device!: GPUDevice;
  private context!: GPUCanvasContext;
  private format!: GPUTextureFormat;
  private canvasSampler!: GPUSampler;
  private renderTarget!: GPUTexture;
  private camera!: Camera;
  private config!: Config;

  private buffers!: Buffers;
  private bindGroupLayouts!: BindGroupLayouts;
  private bindGroups!: BindGroups;
  private pipelines!: Pipelines;

  private frameCount = 0;
  private isDebugMode = false;
  private isProfilingMode = false;
  private querySets: GPUQuerySet[] = [];
  private queryBuffers: GPUBuffer[] = [];
  private readBuffers: GPUBuffer[] = [];
  private slotBusy: boolean[] = [];

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
  }

  async init(): Promise<void> {
    if (!navigator.gpu) throw new Error("WebGPU not supported");

    const adapter = await navigator.gpu.requestAdapter();
    if (!adapter) throw new Error("No GPU adapter found");

    const urlParams = new URLSearchParams(window.location.search);
    this.isDebugMode = urlParams.has("debug");
    this.isProfilingMode =
      urlParams.has("profile") && adapter.features.has("timestamp-query");
    const requiredFeatures: GPUFeatureName[] = this.isProfilingMode
      ? ["timestamp-query"]
      : [];
    this.device = await adapter.requestDevice({ requiredFeatures });

    this.context = this.canvas.getContext("webgpu")!;
    this.format = navigator.gpu.getPreferredCanvasFormat();
    this.canvasSampler = this.device.createSampler({
      magFilter: "nearest",
      minFilter: "nearest",
      mipmapFilter: "nearest",
    });
    this.camera = new Camera(this.device, 0.002, 0.2, vec3.fromValues(8, 8, 8));
    this.config = createConfig(this.device, { max_trace_dist: 50 });

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
      this.isDebugMode,
    );

    this.createProfilingResources();

    const observer = new ResizeObserver(() => {
      clearTimeout(this.resizeDebounce);
      this.resizeDebounce = setTimeout(() => this.resize(), RESIZE_DEBOUNCE_MS);
    });
    observer.observe(this.canvas);
    this.resize();
  }

  update(inputState: InputState): void {
    // this.config.update(this.device.queue, { max_trace_dist: 50 });
    this.camera.update(this.device.queue, inputState);
  }

  frame(): void {
    const ringIdx = this.frameCount % RING_SIZE;
    const slotAvailable = this.isProfilingMode && !this.slotBusy[ringIdx];
    if (this.isProfilingMode && !slotAvailable)
      console.warn(
        `Profiling slot ${ringIdx} still busy, skipping timestamp capture this frame`,
      );
    const qSet = slotAvailable ? this.querySets[ringIdx] : undefined;

    const commandEncoder = this.device.createCommandEncoder();
    this.encodeGenPass(commandEncoder, qSet);
    this.encodeRaytracePass(commandEncoder, qSet);
    this.encodeRenderPass(commandEncoder, qSet);

    if (qSet) {
      const qBuf = this.queryBuffers[ringIdx];
      const rBuf = this.readBuffers[ringIdx];
      commandEncoder.resolveQuerySet(qSet, 0, 6, qBuf, 0);
      commandEncoder.copyBufferToBuffer(qBuf, 0, rBuf, 0, 48);
    }

    this.device.queue.submit([commandEncoder.finish()]);

    if (qSet) {
      this.slotBusy[ringIdx] = true;
      this.readTimestamps(ringIdx)
        .catch((err) =>
          console.error(`Profiling readback failed for slot ${ringIdx}:`, err),
        )
        .finally(() => {
          this.slotBusy[ringIdx] = false;
        });
    }

    this.frameCount++;
  }

  private async readTimestamps(idx: number): Promise<void> {
    if (!this.isProfilingMode) return;

    const rBuf = this.readBuffers[idx];
    await rBuf.mapAsync(GPUMapMode.READ);

    const arrayBuffer = rBuf.getMappedRange();
    const timestamps = new BigInt64Array(arrayBuffer.slice(0));
    rBuf.unmap();

    const genMilliseconds = Number(timestamps[1] - timestamps[0]) / 1_000_000;
    const raytraceMilliseconds =
      Number(timestamps[3] - timestamps[2]) / 1_000_000;
    const renderMilliseconds =
      Number(timestamps[5] - timestamps[4]) / 1_000_000;

    console.log(`
       Gen Pass: ${genMilliseconds.toFixed(4)} ms\n
       Raytrace Pass: ${raytraceMilliseconds.toFixed(4)} ms\n
       Render Pass: ${renderMilliseconds.toFixed(4)} ms
     `);
  }

  private createProfilingResources() {
    if (!this.isProfilingMode) return;

    const capacity = 6;
    for (let i = 0; i < RING_SIZE; i++) {
      this.querySets.push(
        this.device.createQuerySet({ type: "timestamp", count: capacity }),
      );
      this.queryBuffers.push(
        this.device.createBuffer({
          size: 8 * capacity,
          usage: GPUBufferUsage.QUERY_RESOLVE | GPUBufferUsage.COPY_SRC,
        }),
      );
      this.readBuffers.push(
        this.device.createBuffer({
          size: 8 * capacity,
          usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
        }),
      );
      this.slotBusy.push(false);
    }
  }

  private encodeGenPass(
    commandEncoder: GPUCommandEncoder,
    querySet?: GPUQuerySet,
  ): void {
    const pass = commandEncoder.beginComputePass({
      label: "gen pass",
      timestampWrites:
        this.isProfilingMode && querySet
          ? {
              querySet,
              beginningOfPassWriteIndex: 0,
              endOfPassWriteIndex: 1,
            }
          : undefined,
    });
    pass.setPipeline(this.pipelines.gen);
    pass.setBindGroup(0, this.bindGroups.read_write_world);
    pass.dispatchWorkgroups(20, 20, 20);
    pass.end();
  }

  private encodeRaytracePass(
    commandEncoder: GPUCommandEncoder,
    querySet?: GPUQuerySet,
  ): void {
    const pass = commandEncoder.beginComputePass({
      label: "raytrace pass",
      timestampWrites:
        this.isProfilingMode && querySet
          ? {
              querySet,
              beginningOfPassWriteIndex: 2,
              endOfPassWriteIndex: 3,
            }
          : undefined,
    });
    pass.setPipeline(this.pipelines.raytrace);
    pass.setBindGroup(0, this.bindGroups.read_world);
    pass.setBindGroup(1, this.bindGroups.raytrace);
    pass.dispatchWorkgroups(
      Math.ceil(this.canvas.width / 8),
      Math.ceil(this.canvas.height / 8),
    );
    pass.end();
  }

  private encodeRenderPass(
    commandEncoder: GPUCommandEncoder,
    querySet?: GPUQuerySet,
  ): void {
    const canvasTextureView = this.context.getCurrentTexture().createView();
    const renderPass = commandEncoder.beginRenderPass({
      label: "render pass",
      timestampWrites:
        this.isProfilingMode && querySet
          ? {
              querySet,
              beginningOfPassWriteIndex: 4,
              endOfPassWriteIndex: 5,
            }
          : undefined,
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

    if (this.renderTarget) this.renderTarget.destroy();

    this.renderTarget = this.device.createTexture({
      size: [this.canvas.width, this.canvas.height],
      format: "rgba8unorm",
      usage: GPUTextureUsage.STORAGE_BINDING | GPUTextureUsage.TEXTURE_BINDING,
    });

    this.bindGroups = { ...this.bindGroups, ...this.createDynamicBindGroups() };
  }

  private createDynamicBindGroups(): DynamicBindGroups {
    const renderTargetView = this.renderTarget.createView();

    const raytrace = this.device.createBindGroup({
      label: "raytrace bind group",
      layout: this.bindGroupLayouts.raytrace,
      entries: [
        { binding: 0, resource: renderTargetView },
        { binding: 1, resource: { buffer: this.camera.buffer } },
        { binding: 2, resource: { buffer: this.config.buffer } },
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

    return { raytrace, render };
  }
}
