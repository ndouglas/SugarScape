import type { SimLike, SimModule } from './sim-host';
import { Sim } from './wasm-pkg/sugarscape.js';

/** The real WASM `Sim` behind a SimHost; frames are read from `memory`, the instance's memory. */
export function wasmSimModule(memory: WebAssembly.Memory): SimModule {
  return {
    create: (configJson, seed, landscapes): SimLike => new Sim(configJson, seed, landscapes),
    frameBytes: (ptr, len) => new Uint8Array(memory.buffer, ptr, len),
  };
}
