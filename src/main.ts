import "./styles.css";
import { Renderer } from "./Renderer.ts";
import { InputManager } from "./Input.ts";

class GameApp {
  private readonly loadingScreen: HTMLDivElement;
  private readonly inputManager: InputManager;
  private readonly renderer: Renderer;
  private animationFrameId = 0;

  constructor(canvas: HTMLCanvasElement, loadingScreen: HTMLDivElement) {
    this.loadingScreen = loadingScreen;
    this.inputManager = new InputManager(canvas);
    this.renderer = new Renderer(canvas);
  }

  async init() {
    await this.renderer.init();
    this.loadingScreen.style.opacity = "0";
    this.loadingScreen.addEventListener(
      "transitionend",
      () => this.loadingScreen.remove(),
      { once: true },
    );
    this.animationFrameId = requestAnimationFrame(this.gameLoop);
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
