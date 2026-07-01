import "./styles.css";
import { Renderer } from "./core/Renderer.ts";
import { InputManager } from "./core/Input.ts";

class GameApp {
  private readonly canvas: HTMLCanvasElement;
  private readonly loadingScreen: HTMLDivElement;
  private readonly inputManager: InputManager;
  private readonly renderer: Renderer;
  private progressText: HTMLElement;
  private animationFrameId = 0;
  private isPaused = true;

  constructor(canvas: HTMLCanvasElement, loadingScreen: HTMLDivElement) {
    this.canvas = canvas;
    this.loadingScreen = loadingScreen;
    this.inputManager = new InputManager(this.canvas);
    this.renderer = new Renderer(this.canvas);

    const prog = document.getElementById("progress-text");
    if (!(prog instanceof HTMLElement)) {
      throw new Error("Missing #progress-text element");
    }
    this.progressText = prog;
  }

  async init() {
    try {
      await this.renderer.init();
    } catch (e) {
      this.progressText.textContent =
        e instanceof Error ? e.message : "Unknown error";
      return;
    }

    this.progressText.textContent = "Click to Start!";
    this.progressText.classList.add("pulsing");
    document.addEventListener("pointerlockchange", () => {
      if (document.pointerLockElement === this.canvas) {
        this.handleResume();
      } else {
        this.handlePause();
      }
    });
  }

  private handlePause() {
    if (this.isPaused) {
      return;
    }
    this.isPaused = true;

    this.loadingScreen.classList.remove("hidden", "exiting");
    this.progressText.textContent = "Click to Resume";
    this.progressText.classList.remove("pulsing");

    cancelAnimationFrame(this.animationFrameId);
    this.animationFrameId = 0;
  }

  private handleResume() {
    if (!this.isPaused) {
      return;
    }
    this.isPaused = false;

    this.loadingScreen.classList.add("exiting");
    this.loadingScreen.addEventListener(
      "transitionend",
      (e) => {
        if (e.propertyName === "opacity") {
          this.loadingScreen.classList.remove("exiting");
          this.loadingScreen.classList.add("hidden");
        }
      },
      { once: true },
    );

    if (!this.animationFrameId) {
      this.animationFrameId = requestAnimationFrame(this.gameLoop);
    }
  }

  private readonly gameLoop = (_time: number) => {
    const inputState = this.inputManager.poll();
    this.renderer.update(inputState);
    this.renderer.frame();
    this.animationFrameId = requestAnimationFrame(this.gameLoop);
  };

  destroy() {
    cancelAnimationFrame(this.animationFrameId);
  }
}

const canvas = document.getElementById("canvas");
const loadingScreen = document.getElementById("loading-screen");

if (!(canvas instanceof HTMLCanvasElement)) {
  throw new Error("Missing #canvas element");
}
if (!(loadingScreen instanceof HTMLDivElement)) {
  throw new Error("Missing #loading-screen element");
}

new GameApp(canvas, loadingScreen).init();
