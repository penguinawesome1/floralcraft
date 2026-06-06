export interface InputState {
  readonly keys: ReadonlySet<string>;
  readonly deltaX: number;
  readonly deltaY: number;
}

export class InputManager {
  private canvas: HTMLCanvasElement;
  private keys = new Set<string>();
  private deltaX = 0;
  private deltaY = 0;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.initListeners();
  }

  private initListeners() {
    window.addEventListener("keydown", (e) => {
      if (document.pointerLockElement === this.canvas) {
        this.keys.add(e.code);
      }
    });

    window.addEventListener("keyup", (e) => this.keys.delete(e.code));

    window.addEventListener("mousemove", (e) => {
      if (document.pointerLockElement === this.canvas) {
        this.deltaX -= e.movementX;
        this.deltaY -= e.movementY;
      }
    });

    this.canvas.addEventListener("click", async () => {
      if (document.pointerLockElement !== this.canvas) {
        await this.canvas.requestPointerLock();
      }
    });

    window.addEventListener("blur", () => {
      this.keys.clear();
    });
  }

  public poll(): InputState {
    const state: InputState = {
      keys: this.keys,
      deltaX: this.deltaX,
      deltaY: this.deltaY,
    };

    this.deltaX = 0;
    this.deltaY = 0;

    return state;
  }
}
