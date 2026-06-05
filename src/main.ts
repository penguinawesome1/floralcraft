import { Renderer } from "./Renderer.ts";

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const keys = new Set<string>();
let deltaX = 0;
let deltaY = 0;

window.addEventListener("keydown", (e) => {
  if (document.pointerLockElement == canvas) {
    keys.add(e.code);
  }
});
window.addEventListener("keyup", (e) => keys.delete(e.code));
window.addEventListener("mousemove", (e) => {
  if (document.pointerLockElement === canvas) {
    deltaX -= e.movementX;
    deltaY -= e.movementY;
  }
});
canvas.addEventListener("click", async () => {
  if (document.pointerLockElement !== canvas) {
    await canvas.requestPointerLock();
  }
});
window.addEventListener("blur", () => {
  keys.clear();
});

const renderer = new Renderer(canvas);
await renderer.init();

function loop() {
  renderer.update(keys, deltaX, deltaY);

  deltaX = 0;
  deltaY = 0;

  renderer.frame();
  requestAnimationFrame(loop);
}
requestAnimationFrame(loop);
