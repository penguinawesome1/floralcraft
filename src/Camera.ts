export class Camera {
  private pos = [0, 0, 0];
  private pitch = 0;
  private yaw = 0;
  private rotation: Float32Array;
  private sensitivity: number;
  private speed: number;
  private _buffer: GPUBuffer;

  constructor(device: GPUDevice, sensitivity: number, speed: number) {
    this.sensitivity = sensitivity;
    this.speed = speed;
    this._buffer = device.createBuffer({
      size: 80,
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });
    this.rotation = this.buildRotation();
  }

  get buffer(): GPUBuffer {
    return this._buffer;
  }

  update(queue: GPUQueue, keys: Set<string>, deltaX: number, deltaY: number) {
    this.yaw += deltaX * this.sensitivity;
    this.pitch += deltaY * this.sensitivity;
    this.pitch = Math.max(-Math.PI / 2, Math.min(Math.PI / 2, this.pitch));
    this.rotation = this.buildRotation();

    const forward = normalize([this.rotation[6], 0, this.rotation[8]]);
    const right = normalize([this.rotation[0], 0, this.rotation[2]]);

    if (keys.has("KeyW") || keys.has("ArrowUp")) {
      this.pos[0] += forward[0] * this.speed;
      this.pos[1] += forward[1] * this.speed;
      this.pos[2] += forward[2] * this.speed;
    }
    if (keys.has("KeyS") || keys.has("ArrowDown")) {
      this.pos[0] -= forward[0] * this.speed;
      this.pos[1] -= forward[1] * this.speed;
      this.pos[2] -= forward[2] * this.speed;
    }
    if (keys.has("KeyA") || keys.has("ArrowLeft")) {
      this.pos[0] -= right[0] * this.speed;
      this.pos[1] -= right[1] * this.speed;
      this.pos[2] -= right[2] * this.speed;
    }
    if (keys.has("KeyD") || keys.has("ArrowRight")) {
      this.pos[0] += right[0] * this.speed;
      this.pos[1] += right[1] * this.speed;
      this.pos[2] += right[2] * this.speed;
    }

    if (keys.has("Space")) this.pos[1] += this.speed;
    if (keys.has("ShiftLeft")) this.pos[1] -= this.speed;

    queue.writeBuffer(this._buffer, 0, this.toUniform());
  }

  toUniform(): Float32Array {
    return new Float32Array([
      ...this.pos,
      0,
      ...this.rotation.slice(0, 3),
      0,
      ...this.rotation.slice(3, 6),
      0,
      ...this.rotation.slice(6, 9),
      0,
    ]);
  }

  private buildRotation(): Float32Array {
    const f = normalize([
      Math.cos(this.pitch) * Math.sin(this.yaw),
      Math.sin(this.pitch),
      -Math.cos(this.pitch) * Math.cos(this.yaw),
    ]);
    const r = normalize(cross(f, [0, 1, 0]));
    const u = cross(r, f);
    return new Float32Array([...r, ...u, ...f]);
  }
}

const cross = (a: number[], b: number[]): number[] => [
  a[1] * b[2] - a[2] * b[1],
  a[2] * b[0] - a[0] * b[2],
  a[0] * b[1] - a[1] * b[0],
];

const normalize = (a: number[]): number[] => {
  const len = Math.sqrt(a[0] ** 2 + a[1] ** 2 + a[2] ** 2);
  return a.map((v) => v / len);
};
