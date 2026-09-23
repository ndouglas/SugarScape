import { defineConfig } from 'vitest/config';

export default defineConfig({
  // Relative asset URLs so the build works under a GitHub Pages sub-path.
  base: './',
  // Sweep workers import the wasm-bindgen module, which finds its .wasm with
  // new URL(…, import.meta.url): build workers as ES modules.
  worker: { format: 'es' },
  test: { environment: 'node', include: ['src/**/*.test.ts'] },
});
