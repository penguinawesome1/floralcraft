import { vec3, mat4 } from "gl-matrix";
import type { InputState } from "./Input";

export class Camera {
  private pos: vec3 = vec3.create();
  private pitch = 0;
  private yaw = 0;
  private rotation: mat4 = mat4.create();
  private readonly sensitivity: number;
  private readonly speed: number;
  private _buffer: GPUBuffer;
  private uniformData = new Float32Array(20);

  constructor(
    device: GPUDevice,
    sensitivity: number,
    speed: number,
    initialPos: vec3 = vec3.fromValues(0, 0, 0),
  ) {
    vec3.copy(this.pos, initialPos);
    this.sensitivity = sensitivity;
    this.speed = speed;
    this._buffer = device.createBuffer({
      label: "camera buffer",
      size: 80,
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });

    this.updateRotation();
  }

  get buffer(): GPUBuffer {
    return this._buffer;
  }

  update(
    queue: GPUQueue,
    deltaTime: number,
    { keys, deltaX, deltaY }: InputState,
  ) {
    this.yaw += deltaX * this.sensitivity;
    this.pitch += deltaY * this.sensitivity;
    const MAX_PITCH = Math.PI / 2;
    this.pitch = Math.max(-MAX_PITCH, Math.min(MAX_PITCH, this.pitch));
    this.updateRotation();

    const moveDir = vec3.create();
    const backward = this.getBackward(vec3.create());
    const right = this.getRight(vec3.create());

    if (keys.has("KeyW") || keys.has("ArrowUp"))
      vec3.sub(moveDir, moveDir, backward);
    if (keys.has("KeyS") || keys.has("ArrowDown"))
      vec3.add(moveDir, moveDir, backward);
    if (keys.has("KeyA") || keys.has("ArrowLeft"))
      vec3.sub(moveDir, moveDir, right);
    if (keys.has("KeyD") || keys.has("ArrowRight"))
      vec3.add(moveDir, moveDir, right);

    if (keys.has("Space")) moveDir[1] += 1;
    if (keys.has("ShiftLeft")) moveDir[1] -= 1;

    if (vec3.length(moveDir) > 0) {
      vec3.normalize(moveDir, moveDir);
      vec3.scaleAndAdd(this.pos, this.pos, moveDir, this.speed * deltaTime);
    }

    this.updateUniform();
    queue.writeBuffer(this._buffer, 0, this.uniformData);
  }

  private updateRotation() {
    mat4.identity(this.rotation);
    mat4.rotateY(this.rotation, this.rotation, this.yaw);
    mat4.rotateX(this.rotation, this.rotation, this.pitch);
  }

  private getBackward(out: vec3): vec3 {
    vec3.set(out, 0, 0, 1);
    vec3.transformMat4(out, out, this.rotation);
    out[1] = 0;
    return vec3.normalize(out, out);
  }

  private getRight(out: vec3): vec3 {
    vec3.set(out, 1, 0, 0);
    vec3.transformMat4(out, out, this.rotation);
    out[1] = 0;
    return vec3.normalize(out, out);
  }

  private updateUniform() {
    this.uniformData.set(this.pos, 0);
    this.uniformData[3] = 0; // padding
    this.uniformData.set(this.rotation, 4);
  }
}
