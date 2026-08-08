export interface InputState {
  readonly keys: ReadonlySet<string>;
  readonly deltaX: number;
  readonly deltaY: number;
  readonly pendingAction: number;
}

export class InputManager {
  private canvas: HTMLCanvasElement;
  private keys = new Set<string>();
  private deltaX = 0;
  private deltaY = 0;
  private pendingAction = 0;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.initListeners();
  }

  private initListeners() {
    window.addEventListener("keydown", (e) => {
      if (document.pointerLockElement !== this.canvas) return;
      this.keys.add(e.code);
    });

    window.addEventListener("keyup", (e) => this.keys.delete(e.code));

    window.addEventListener("mousemove", (e) => {
      if (document.pointerLockElement !== this.canvas) return;
      this.deltaX -= e.movementX;
      this.deltaY -= e.movementY;
    });

    this.canvas.addEventListener("click", async () => {
      if (document.pointerLockElement === this.canvas) return;
      await this.canvas.requestPointerLock();
    });

    window.addEventListener("blur", () => {
      this.keys.clear();
    });

    this.canvas.addEventListener("mousedown", (e) => {
      if (document.pointerLockElement !== this.canvas) return;
      if (e.button === 0)
        this.pendingAction = 1; // break
      else if (e.button === 2) this.pendingAction = 2; // place
    });

    this.canvas.addEventListener("contextmenu", (e) => e.preventDefault());
  }

  public poll(): InputState {
    const state: InputState = {
      keys: this.keys,
      deltaX: this.deltaX,
      deltaY: this.deltaY,
      pendingAction: this.pendingAction,
    };

    this.deltaX = 0;
    this.deltaY = 0;
    this.pendingAction = 0;

    return state;
  }
}
