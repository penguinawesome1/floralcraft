export class Clock {
  private lastTime = performance.now();
  elapsedSeconds = 0;

  update(): number {
    const now = performance.now();
    const deltaTime = (now - this.lastTime) / 1000;
    this.lastTime = now;
    this.elapsedSeconds += deltaTime;
    return deltaTime;
  }
}
