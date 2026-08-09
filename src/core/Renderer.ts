import { Camera } from "../player/Camera.ts";
import type { InputState } from "../player/Input.ts";
import {
  type BindGroupLayouts,
  createBindGroupLayouts,
} from "../gpu/BindGroupLayouts.ts";
import {
  type BindGroups,
  createDynamicBindGroups,
  createStaticBindGroups,
} from "../gpu/BindGroups.ts";
import { type Resources, createResources } from "../gpu/Resources.ts";
import { type Pipelines, createPipelines } from "../gpu/Pipelines.ts";
import {
  type Config,
  createConfig,
  GEN_SIDE,
  packInputFlags,
  CHUNK_SIDE,
} from "./Config.ts";
import type { Item } from "../player/Item.ts";
import { FrameEncoder } from "./FrameEncoder.ts";

const CAPPED_MAX_TRACE_DIST = (GEN_SIDE / 2 - 1) * CHUNK_SIDE;
const RING_SIZE = 10;
const RESIZE_DEBOUNCE_MS = 200;

export class Renderer {
  private readonly canvas: HTMLCanvasElement;
  private readonly camera = new Camera(0.002);
  private readonly timestamps = new BigInt64Array(10);
  private resizeDebounce: ReturnType<typeof setTimeout> | undefined;
  private device!: GPUDevice;
  private context!: GPUCanvasContext;
  private format!: GPUTextureFormat;
  private canvasSampler!: GPUSampler;
  private renderTarget!: GPUTexture;
  private config!: Config;
  private frameEncoder!: FrameEncoder;
  private maxTraceDist = CAPPED_MAX_TRACE_DIST;

  private resources!: Resources;
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

  async init(heldItem: Item): Promise<void> {
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
    this.device = await adapter.requestDevice({
      requiredFeatures,
      requiredLimits: {
        maxBufferSize: adapter.limits.maxBufferSize,
        maxStorageBufferBindingSize: adapter.limits.maxStorageBufferBindingSize,
      },
    });

    this.context = this.canvas.getContext("webgpu")!;
    this.format = navigator.gpu.getPreferredCanvasFormat();
    this.canvasSampler = this.device.createSampler({
      magFilter: "nearest",
      minFilter: "nearest",
      mipmapFilter: "nearest",
    });

    this.config = createConfig(this.device, {
      camRotation: this.camera.rotation,
      camYaw: this.camera.yaw,
      maxTraceDist: this.maxTraceDist,
      timeOfDay: 0.5,
      deltaTime: 0.0,
      pendingAction: 0,
      heldItem,
      inputFlags: 0,
    });

    this.bindGroupLayouts = createBindGroupLayouts(this.device);
    this.resources = createResources(this.device);
    const staticBindGroups = createStaticBindGroups(
      this.device,
      this.bindGroupLayouts,
      this.resources,
      this.config,
    );
    this.bindGroups = { ...staticBindGroups } as BindGroups;
    this.pipelines = await createPipelines(
      this.device,
      this.format,
      this.bindGroupLayouts,
      this.isDebugMode,
    );

    this.createProfilingResources();

    this.frameEncoder = new FrameEncoder(
      this.pipelines,
      this.bindGroups,
      this.resources,
      this.context,
      this.isProfilingMode,
    );

    const observer = new ResizeObserver(() => {
      clearTimeout(this.resizeDebounce);
      this.resizeDebounce = setTimeout(() => this.resize(), RESIZE_DEBOUNCE_MS);
    });
    observer.observe(this.canvas);
    this.resize();
  }

  update(
    inputState: InputState,
    heldItem: Item,
    deltaTime: number,
    timeOfDay: number,
  ): void {
    if (inputState.keys.has("BracketLeft")) {
      this.maxTraceDist /= 1.05;
      this.maxTraceDist = Math.max(50, this.maxTraceDist);
    }
    if (inputState.keys.has("BracketRight")) {
      this.maxTraceDist *= 1.05;
      this.maxTraceDist = Math.min(CAPPED_MAX_TRACE_DIST, this.maxTraceDist);
    }

    this.camera.update(inputState.deltaX, inputState.deltaY);

    this.config.update(this.device.queue, {
      camRotation: this.camera.rotation,
      camYaw: this.camera.yaw,
      timeOfDay,
      maxTraceDist: this.maxTraceDist,
      deltaTime,
      pendingAction: inputState.pendingAction,
      heldItem,
      inputFlags: packInputFlags(inputState.keys),
    });
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
    this.frameEncoder.encode(
      commandEncoder,
      this.canvas.width,
      this.canvas.height,
      qSet,
    );

    if (qSet) {
      const qBuf = this.queryBuffers[ringIdx];
      const rBuf = this.readBuffers[ringIdx];
      const capacity = 10;
      commandEncoder.resolveQuerySet(qSet, 0, capacity, qBuf, 0);
      commandEncoder.copyBufferToBuffer(qBuf, 0, rBuf, 0, capacity * 8);
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

  private createProfilingResources(): void {
    if (!this.isProfilingMode) return;

    const capacity = 10;
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

  private async readTimestamps(idx: number): Promise<void> {
    if (!this.isProfilingMode) return;

    const rBuf = this.readBuffers[idx];
    await rBuf.mapAsync(GPUMapMode.READ);

    const mappedView = new BigInt64Array(rBuf.getMappedRange());
    this.timestamps.set(mappedView);
    rBuf.unmap();

    const unloadReclaimMs =
      Number(this.timestamps[1] - this.timestamps[0]) / 1_000_000;
    const genMs = Number(this.timestamps[3] - this.timestamps[2]) / 1_000_000;
    const mipMs = Number(this.timestamps[5] - this.timestamps[4]) / 1_000_000;
    const renderMs =
      Number(this.timestamps[7] - this.timestamps[6]) / 1_000_000;
    const presentMs =
      Number(this.timestamps[9] - this.timestamps[8]) / 1_000_000;

    console.log(`
       Unload Reclaim Pass: ${unloadReclaimMs.toFixed(4)} ms\n
       Gen Pass: ${genMs.toFixed(4)} ms\n
       Mip Pass: ${mipMs.toFixed(4)} ms\n
       Render Pass: ${renderMs.toFixed(4)} ms\n
       Present Pass: ${presentMs.toFixed(4)} ms
     `);
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

    this.bindGroups = {
      ...this.bindGroups,
      ...createDynamicBindGroups(
        this.device,
        this.bindGroupLayouts,
        this.renderTarget,
        this.canvasSampler,
      ),
    };

    this.frameEncoder = new FrameEncoder(
      this.pipelines,
      this.bindGroups,
      this.resources,
      this.context,
      this.isProfilingMode,
    );
  }
}
