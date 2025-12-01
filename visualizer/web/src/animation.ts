import { getVisualizer } from "./wasm";

interface Visualizer {
  tick: () => void;
  set_pattern: (pattern: string) => void;
  enable_oscillating_rate: (amplitude: number, period: number) => void;
  disable_oscillating_rate: () => void;
  read_leds_into: (buffer: Uint8Array) => void;
  spines: () => number;
  spine_len: (idx: number) => number;
  tip_len: (idx: number) => number;
  arc_len: (idx: number) => number;
  total_leds: () => number;
}

export interface Hsv {
  h: number;
  s: number;
  v: number;
}

export interface AnimationState {
  leds: Hsv[];
  spines: number;
  spineLen: (idx: number) => number;
  tipLen: (idx: number) => number;
  arcLen: (idx: number) => number;
}

type StateCallback = (state: AnimationState) => void;

class AnimationManager {
  private visualizer: Visualizer | null = null;
  private ledBuffer: Uint8Array | null = null;
  private leds: Hsv[] = [];
  private running = false;
  private speed = 30; // FPS
  private subscribers: Set<StateCallback> = new Set();
  private animationState: AnimationState | null = null;

  async initialize() {
    if (this.visualizer) return;

    this.visualizer = (await getVisualizer()) as unknown as Visualizer;
    const totalLeds = this.visualizer.total_leds();
    this.ledBuffer = new Uint8Array(totalLeds * 3);

    // Pre-allocate HSV objects
    this.leds = new Array(totalLeds);
    for (let i = 0; i < totalLeds; i++) {
      this.leds[i] = { h: 0, s: 0, v: 0 };
    }

    this.running = true;
    this.animationState = this.getCurrentState();

    console.log("init");
    this.animate();
  }

  start() {
    this.running = true;
  }

  stop() {
    this.running = false;
  }

  setSpeed(fps: number) {
    this.speed = Math.max(1, Math.min(60, fps));
  }

  async setPattern(pattern: string) {
    if (!this.visualizer) await this.initialize();
    this.visualizer!.set_pattern(pattern);

    const state = this.getCurrentState();
    this.animationState = state;
  }

  async enableOscillatingRate(amplitude: number, period: number) {
    if (!this.visualizer) await this.initialize();
    this.visualizer!.enable_oscillating_rate(amplitude, period);
  }

  async disableOscillatingRate() {
    if (!this.visualizer) await this.initialize();
    this.visualizer!.disable_oscillating_rate();
  }

  subscribe(callback: StateCallback): () => void {
    this.subscribers.add(callback);
    // Immediately call with current state
    if (this.visualizer) {
      callback(this.animationState!);
    }
    return () => {
      this.subscribers.delete(callback);
    };
  }

  private animate = () => {
    if (this.running && this.visualizer && this.ledBuffer) {
      // Tick the visualizer
      this.visualizer.tick();

      // Read LED colors into buffer (zero-copy)
      this.visualizer.read_leds_into(this.ledBuffer);

      // Update HSV objects in-place
      const totalLeds = this.visualizer.total_leds();
      for (let i = 0; i < totalLeds; i++) {
        const idx = i * 3;
        this.leds[i].h = this.ledBuffer[idx];
        this.leds[i].s = this.ledBuffer[idx + 1];
        this.leds[i].v = this.ledBuffer[idx + 2];
      }

      // Defer subscriber notifications to avoid re-entrancy
      const state = this.animationState!;
      requestAnimationFrame(() => {
        this.subscribers.forEach((callback) => callback(state));
      });
    }

    // Schedule next frame
    setTimeout(() => {
      this.animate();
    }, 1000 / this.speed);
  };

  private getCurrentState(): AnimationState {
    return {
      leds: this.leds,
      spines: this.visualizer!.spines(),
      spineLen: (idx) => this.visualizer!.spine_len(idx),
      tipLen: (idx) => this.visualizer!.tip_len(idx),
      arcLen: (idx) => this.visualizer!.arc_len(idx),
    };
  }
}

// Singleton instance
export const animationManager = new AnimationManager();
