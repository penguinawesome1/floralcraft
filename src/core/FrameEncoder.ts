import type { BindGroups } from "../gpu/BindGroups";
import type { Pipelines } from "../gpu/Pipelines";
import type { Resources } from "../gpu/Resources";
import { GEN_SIDE } from "./Config";

export class FrameEncoder {
  private readonly pipelines: Pipelines;
  private readonly bindGroups: BindGroups;
  private readonly resources: Resources;
  private readonly context: GPUCanvasContext;
  private readonly isProfilingMode: boolean;

  constructor(
    pipelines: Pipelines,
    bindGroups: BindGroups,
    resources: Resources,
    context: GPUCanvasContext,
    isProfilingMode: boolean,
  ) {
    this.pipelines = pipelines;
    this.bindGroups = bindGroups;
    this.resources = resources;
    this.context = context;
    this.isProfilingMode = isProfilingMode;
  }

  encode(
    commandEncoder: GPUCommandEncoder,
    canvasWidth: number,
    canvasHeight: number,
    querySet?: GPUQuerySet,
  ) {
    this.movementPass(commandEncoder);
    this.unloadReclaimPass(commandEncoder, querySet);
    this.genPass(commandEncoder, querySet);
    this.mipPass(commandEncoder, querySet);
    commandEncoder.clearBuffer(this.resources.gen_flags);
    commandEncoder.clearBuffer(this.resources.load_list);
    this.renderPass(commandEncoder, canvasWidth, canvasHeight, querySet);
    this.modifyStorePass(commandEncoder);
    this.presentPass(commandEncoder, querySet);
  }

  private movementPass(commandEncoder: GPUCommandEncoder): void {
    const pass = commandEncoder.beginComputePass({ label: "movement pass" });
    pass.setPipeline(this.pipelines.movement);
    pass.setBindGroup(0, this.bindGroups.movement);
    pass.dispatchWorkgroups(1, 1, 1);
    pass.end();
  }

  private unloadReclaimPass(
    commandEncoder: GPUCommandEncoder,
    querySet?: GPUQuerySet,
  ): void {
    const pass = commandEncoder.beginComputePass({
      label: "unload reclaim pass",
      timestampWrites: this.timestampWrites(querySet, 0, 1),
    });

    pass.setPipeline(this.pipelines.prepUnload);
    pass.setBindGroup(0, this.bindGroups.prepUnload);
    pass.dispatchWorkgroups(1, 1, 1);

    pass.setPipeline(this.pipelines.free);
    pass.setBindGroup(0, this.bindGroups.free);
    pass.dispatchWorkgroupsIndirect(this.resources.indirect_args, 0);

    pass.setPipeline(this.pipelines.clear);
    pass.setBindGroup(0, this.bindGroups.clear);
    pass.dispatchWorkgroupsIndirect(this.resources.indirect_args, 0);

    pass.setPipeline(this.pipelines.compact);
    pass.setBindGroup(0, this.bindGroups.compact);
    const totalWords = Math.ceil(GEN_SIDE ** 3 / 32);
    pass.dispatchWorkgroups(Math.ceil(totalWords / 128), 1, 1);

    pass.setPipeline(this.pipelines.prepGen);
    pass.setBindGroup(0, this.bindGroups.prepGen);
    pass.dispatchWorkgroups(1, 1, 1);

    pass.end();
  }

  private genPass(
    commandEncoder: GPUCommandEncoder,
    querySet?: GPUQuerySet,
  ): void {
    const pass = commandEncoder.beginComputePass({
      label: "gen pass",
      timestampWrites: this.timestampWrites(querySet, 2, 3),
    });
    pass.setPipeline(this.pipelines.gen);
    pass.setBindGroup(0, this.bindGroups.gen);
    pass.dispatchWorkgroupsIndirect(this.resources.indirect_args, 0);
    pass.end();
  }

  private mipPass(
    commandEncoder: GPUCommandEncoder,
    querySet?: GPUQuerySet,
  ): void {
    const pass = commandEncoder.beginComputePass({
      label: "mip pass",
      timestampWrites: this.timestampWrites(querySet, 4, 5),
    });
    pass.setPipeline(this.pipelines.mip);
    pass.setBindGroup(0, this.bindGroups.mip);
    pass.dispatchWorkgroupsIndirect(this.resources.indirect_args, 12);
    pass.end();
  }

  private renderPass(
    commandEncoder: GPUCommandEncoder,
    canvasWidth: number,
    canvasHeight: number,
    querySet?: GPUQuerySet,
  ): void {
    const pass = commandEncoder.beginComputePass({
      label: "render pass",
      timestampWrites: this.timestampWrites(querySet, 6, 7),
    });
    pass.setPipeline(this.pipelines.render);
    pass.setBindGroup(0, this.bindGroups.renderStatic);
    pass.setBindGroup(1, this.bindGroups.renderDynamic);
    pass.dispatchWorkgroups(
      Math.ceil(canvasWidth / 8),
      Math.ceil(canvasHeight / 8),
    );
    pass.end();
  }

  private modifyStorePass(commandEncoder: GPUCommandEncoder): void {
    const pass = commandEncoder.beginComputePass({
      label: "modify store pass",
    });

    pass.setPipeline(this.pipelines.modify);
    pass.setBindGroup(0, this.bindGroups.modify);
    pass.dispatchWorkgroups(1, 1, 1);

    pass.setBindGroup(0, this.bindGroups.store);
    pass.setPipeline(this.pipelines.store);
    pass.dispatchWorkgroups(1, 1, 1);

    pass.end();
  }

  private presentPass(
    commandEncoder: GPUCommandEncoder,
    querySet?: GPUQuerySet,
  ): void {
    const canvasTextureView = this.context.getCurrentTexture().createView();
    const pass = commandEncoder.beginRenderPass({
      label: "present pass",
      timestampWrites: this.timestampWrites(querySet, 8, 9),
      colorAttachments: [
        {
          view: canvasTextureView,
          loadOp: "load",
          storeOp: "store",
        },
      ],
    });
    pass.setPipeline(this.pipelines.present);
    pass.setBindGroup(0, this.bindGroups.present);
    pass.draw(3);
    pass.end();
  }

  private timestampWrites(
    querySet: GPUQuerySet | undefined,
    start: number,
    end: number,
  ) {
    return this.isProfilingMode && querySet
      ? { querySet, beginningOfPassWriteIndex: start, endOfPassWriteIndex: end }
      : undefined;
  }
}
