// Singleton WASM loader to prevent memory leaks on HMR
let wasmModule: any | null = null;
let visualizerInstance: any | null = null;
let currentPatterns: string[] | null = null;

export async function getWasm() {
  if (!wasmModule) {
    const wasm = await import("../pkg/led_star_visualizer");
    await wasm.default();
    wasmModule = wasm;
  }
  return wasmModule;
}

export async function getVisualizer() {
  if (!visualizerInstance) {
    const wasm = await getWasm();
    visualizerInstance = new wasm.Visualizer();
  }
  return visualizerInstance;
}

export async function getPatterns() {
  if (currentPatterns) return currentPatterns;
  const wasm = await getWasm();
  currentPatterns = wasm.get_available_patterns();
  return currentPatterns;
}
