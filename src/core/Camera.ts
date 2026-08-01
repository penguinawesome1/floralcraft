import { mat4 } from "gl-matrix";

export class Camera {
  private pitch = 0;
  private _yaw = 0;
  private _rotation: mat4 = mat4.create();
  private readonly sensitivity: number;

  constructor(sensitivity: number) {
    this.sensitivity = sensitivity;
    this.updateRotation();
  }

  get yaw(): number {
    return this._yaw;
  }

  get rotation(): mat4 {
    return this._rotation;
  }

  update(deltaX: number, deltaY: number) {
    this._yaw += deltaX * this.sensitivity;
    this.pitch += deltaY * this.sensitivity;
    const MAX_PITCH = Math.PI / 2;
    this.pitch = Math.max(-MAX_PITCH, Math.min(MAX_PITCH, this.pitch));
    this.updateRotation();
  }

  private updateRotation() {
    mat4.identity(this._rotation);
    mat4.rotateY(this._rotation, this._rotation, this._yaw);
    mat4.rotateX(this._rotation, this._rotation, this.pitch);
  }
}
