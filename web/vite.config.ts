import { readFileSync } from 'node:fs';
import type { Plugin } from 'vite';
import { defineConfig } from 'vitest/config';

/**
 * The Long House Valley data compiled into the WASM are GPL-2.0: serve their
 * notice, license and citation beside the page as `anasazi-data/<file>`,
 * copied from `data/anasazi/` (the Rules panel links the notice).
 */
function anasaziNotice(): Plugin {
  const files = ['NOTICE', 'LICENSE', 'CITATION.cff'];
  const read = (name: string) => readFileSync(new URL(`../data/anasazi/${name}`, import.meta.url));
  return {
    name: 'anasazi-notice',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        // (No Node types in this project: the request is read as having a URL.)
        const url = (req as { url?: string }).url ?? '';
        const name = files.find((f) => url.split('?')[0].endsWith(`/anasazi-data/${f}`));
        if (!name) return next();
        res.setHeader('Content-Type', 'text/plain; charset=utf-8');
        res.end(read(name));
      });
    },
    generateBundle() {
      for (const name of files) this.emitFile({ type: 'asset', fileName: `anasazi-data/${name}`, source: read(name) });
    },
  };
}

export default defineConfig({
  // Relative asset URLs so the build works under a GitHub Pages sub-path.
  base: './',
  plugins: [anasaziNotice()],
  // Sweep workers import the wasm-bindgen module, which finds its .wasm with
  // new URL(…, import.meta.url): build workers as ES modules.
  worker: { format: 'es' },
  test: { environment: 'node', include: ['src/**/*.test.ts'] },
});
