import "./styles.css";
import { Renderer } from "./core/Renderer.ts";
import { InputManager } from "./core/Input.ts";
import thanksgivingLoveUrl from "./assets/audio/thanksgiving-love.m4a";
import nightTrackUrl from "./assets/audio/12-am.m4a";

class GameApp {
  private readonly canvas: HTMLCanvasElement;
  private readonly loadingScreen: HTMLDivElement;
  private readonly inputManager: InputManager;
  private readonly renderer: Renderer;
  private readonly daytimeTrack: HTMLAudioElement;
  private readonly nighttimeTrack: HTMLAudioElement;
  private currentTrack: HTMLAudioElement | null = null;
  private isFading = false;
  private progressText: HTMLElement;
  private animationFrameId = 0;
  private isPaused = true;

  constructor(canvas: HTMLCanvasElement, loadingScreen: HTMLDivElement) {
    this.canvas = canvas;
    this.loadingScreen = loadingScreen;
    this.inputManager = new InputManager(this.canvas);
    this.renderer = new Renderer(this.canvas);

    this.daytimeTrack = new Audio(thanksgivingLoveUrl);
    this.daytimeTrack.loop = true;
    this.nighttimeTrack = new Audio(nightTrackUrl);
    this.nighttimeTrack.loop = true;
    this.currentTrack = this.daytimeTrack;

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

    if (this.currentTrack) this.currentTrack.volume = 0.25;

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

    if (this.currentTrack) {
      this.currentTrack.play().catch((err) => {
        console.warn("Audio playback prevented:", err);
      });
      this.currentTrack.volume = 0.5;
    }

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

  private switchTrack(targetTrack: HTMLAudioElement) {
    if (this.currentTrack === targetTrack || this.isFading) return;

    this.isFading = true;
    const oldTrack = this.currentTrack as HTMLAudioElement;

    const fadeInterval = setInterval(() => {
      if (oldTrack.volume > 0.05) {
        oldTrack.volume -= 0.05;
        return;
      }

      clearInterval(fadeInterval);
      oldTrack.pause();
      oldTrack.currentTime = 0;

      this.currentTrack = targetTrack;
      this.currentTrack.volume = this.isPaused ? 0.2 : 0.5;

      if (!this.isPaused) {
        this.currentTrack.play().catch((err) => console.warn(err));
      }

      this.isFading = false;
    }, 1000);
  }

  private readonly gameLoop = (_time: number) => {
    const inputState = this.inputManager.poll();
    this.renderer.update(inputState);

    const timeOfDay = this.renderer.getTimeOfDay();
    const target =
      timeOfDay > 0.25 && timeOfDay < 0.75
        ? this.daytimeTrack
        : this.nighttimeTrack;
    this.switchTrack(target);

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
