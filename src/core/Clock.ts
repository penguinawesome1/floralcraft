const MAX_DELTA = 0.1;

export class Clock {
  private lastTime = performance.now();
  elapsedSeconds = 0;

  update(): number {
    const now = performance.now();
    let deltaTime = (now - this.lastTime) / 1000;
    deltaTime = Math.min(deltaTime, MAX_DELTA);
    this.lastTime = now;
    this.elapsedSeconds += deltaTime;
    return deltaTime;
  }
}
