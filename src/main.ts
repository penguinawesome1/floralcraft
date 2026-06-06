import { Renderer } from "./Renderer.ts";
import { InputManager } from "./Input.ts";

const canvas = document.getElementById("canvas") as HTMLCanvasElement;

const loadingScreen = document.getElementById("loading-screen");

const inputManager = new InputManager(canvas);
const renderer = new Renderer(canvas);
await renderer.init();

fadeLoadingScreen();
requestAnimationFrame(loop);

function loop() {
  let input_state = inputManager.poll();
  renderer.update(input_state);
  renderer.frame();
  requestAnimationFrame(loop);
}

function fadeLoadingScreen() {
  if (!loadingScreen) return;

  loadingScreen.style.opacity = "0";

  setTimeout(() => {
    loadingScreen.remove();
  }, 500);
}
