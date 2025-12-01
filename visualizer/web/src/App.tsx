import { useEffect, useRef, useState } from "react";
import "./App.css";
import { getPatterns } from "./wasm";
import { animationManager, AnimationState, Hsv } from "./animation";

function hsvToRgb(hsv: Hsv): { r: number; g: number; b: number } {
  const h = hsv.h / 255;
  const s = hsv.s / 255;
  const v = hsv.v / 255;

  const i = Math.floor(h * 6);
  const f = h * 6 - i;
  const p = v * (1 - s);
  const q = v * (1 - f * s);
  const t = v * (1 - (1 - f) * s);

  let r: number, g: number, b: number;

  switch (i % 6) {
    case 0:
      (r = v), (g = t), (b = p);
      break;
    case 1:
      (r = q), (g = v), (b = p);
      break;
    case 2:
      (r = p), (g = v), (b = t);
      break;
    case 3:
      (r = p), (g = q), (b = v);
      break;
    case 4:
      (r = t), (g = p), (b = v);
      break;
    case 5:
      (r = v), (g = p), (b = q);
      break;
    default:
      (r = 0), (g = 0), (b = 0);
  }

  return {
    r: Math.round(r * 255),
    g: Math.round(g * 255),
    b: Math.round(b * 255),
  };
}

function App() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [patterns, setPatterns] = useState<string[]>([]);
  const [currentPattern, setCurrentPattern] = useState<string>("Solid");
  const [isPlaying, setIsPlaying] = useState(true);
  const [speed, setSpeed] = useState(30); // FPS

  // Initialize animation manager and load patterns
  useEffect(() => {
    async function initialize() {
      try {
        await animationManager.initialize();
        const availablePatterns = await getPatterns();
        setPatterns(availablePatterns as unknown as string[]);
      } catch (error) {
        console.error("Failed to initialize:", error);
      }
    }
    initialize();
  }, []);

  // Subscribe to animation updates and start render loop
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let rafId: number | null = null;
    let currentState: AnimationState | null = null;

    // Subscribe to state updates
    const unsubscribe = animationManager.subscribe((state) => {
      currentState = state;
    });

    // Render loop - reads from current state
    const render = () => {
      if (currentState) {
        const { leds, spines, spineLen, tipLen, arcLen } = currentState;

        // Clear canvas
        ctx.fillStyle = "#0a0a0a";
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        // Calculate center
        const centerX = canvas.width / 2;
        const centerY = canvas.height / 2;
        const radius = Math.min(canvas.width, canvas.height) * 0.4;

        // Draw star pattern
        let ledIndex = 0;
        for (let spineIdx = 0; spineIdx < spines; spineIdx++) {
          const angle = (spineIdx / spines) * Math.PI * 2 - Math.PI / 2;
          const currentSpineLen = spineLen(spineIdx);
          const currentTipLen = tipLen(spineIdx);
          const currentArcLen = arcLen(spineIdx);

          // Draw spine out
          for (let i = 0; i < currentSpineLen; i++) {
            const progress = (i + 1) / (currentSpineLen + currentTipLen / 2);
            const x = centerX + Math.cos(angle) * radius * progress;
            const y = centerY + Math.sin(angle) * radius * progress;

            const hsv = leds[ledIndex++];
            const rgb = hsvToRgb(hsv);

            ctx.fillStyle = `rgb(${rgb.r}, ${rgb.g}, ${rgb.b})`;
            ctx.beginPath();
            ctx.arc(x, y, 4, 0, Math.PI * 2);
            ctx.fill();
          }

          // Draw tip
          for (let i = 0; i < currentTipLen; i++) {
            const progress = 1.0 + (i + 1) / (currentTipLen * 2);
            const x = centerX + Math.cos(angle) * radius * progress;
            const y = centerY + Math.sin(angle) * radius * progress;

            const hsv = leds[ledIndex++];
            const rgb = hsvToRgb(hsv);

            ctx.fillStyle = `rgb(${rgb.r}, ${rgb.g}, ${rgb.b})`;
            ctx.beginPath();
            ctx.arc(x, y, 5, 0, Math.PI * 2);
            ctx.fill();
          }

          // Draw spine back (mirrored)
          for (let i = currentSpineLen - 1; i >= 0; i--) {
            const progress = (i + 1) / (currentSpineLen + currentTipLen / 2);
            const x = centerX + Math.cos(angle) * radius * progress;
            const y = centerY + Math.sin(angle) * radius * progress;

            const hsv = leds[ledIndex++];
            const rgb = hsvToRgb(hsv);

            ctx.fillStyle = `rgb(${rgb.r}, ${rgb.g}, ${rgb.b})`;
            ctx.beginPath();
            ctx.arc(x, y, 3, 0, Math.PI * 2);
            ctx.fill();
          }

          // Draw arc
          if (currentArcLen > 0) {
            const nextAngle =
              ((spineIdx + 1) / spines) * Math.PI * 2 - Math.PI / 2;
            for (let i = 0; i < currentArcLen; i++) {
              const t = (i + 1) / (currentArcLen + 1);
              const arcAngle = angle + (nextAngle - angle) * t;
              const x = centerX + Math.cos(arcAngle) * radius * 0.7;
              const y = centerY + Math.sin(arcAngle) * radius * 0.7;

              const hsv = leds[ledIndex++];
              const rgb = hsvToRgb(hsv);

              ctx.fillStyle = `rgb(${rgb.r}, ${rgb.g}, ${rgb.b})`;
              ctx.beginPath();
              ctx.arc(x, y, 3, 0, Math.PI * 2);
              ctx.fill();
            }
          }
        }
      }

      rafId = requestAnimationFrame(render);
    };

    render();

    return () => {
      unsubscribe();
      if (rafId !== null) {
        cancelAnimationFrame(rafId);
      }
    };
  }, []);

  // Handle play/pause
  useEffect(() => {
    if (isPlaying) {
      animationManager.start();
    } else {
      animationManager.stop();
    }
  }, [isPlaying]);

  // Handle speed changes
  useEffect(() => {
    animationManager.setSpeed(speed);
  }, [speed]);

  const handlePatternChange = (pattern: string) => {
    setCurrentPattern(pattern);
    animationManager.setPattern(pattern);
  };

  return (
    <div className="app">
      <header className="header">
        <h1>LED Star Visualizer</h1>
        <div className="controls">
          <div className="control-group">
            <label>Pattern:</label>
            <select
              value={currentPattern}
              onChange={(e) => handlePatternChange(e.target.value)}
            >
              {patterns.map((pattern) => (
                <option key={pattern} value={pattern}>
                  {pattern}
                </option>
              ))}
            </select>
          </div>
          <div className="control-group">
            <button onClick={() => setIsPlaying(!isPlaying)}>
              {isPlaying ? "Pause" : "Play"}
            </button>
          </div>
          <div className="control-group">
            <label>Speed: {speed} FPS</label>
            <input
              type="range"
              min="1"
              max="60"
              value={speed}
              onChange={(e) => setSpeed(Number(e.target.value))}
            />
          </div>
        </div>
      </header>
      <main className="canvas-container">
        <canvas
          ref={canvasRef}
          width={800}
          height={800}
          className="visualization-canvas"
        />
      </main>
    </div>
  );
}

export default App;
