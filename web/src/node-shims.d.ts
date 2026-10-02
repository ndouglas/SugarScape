// Node APIs used by Vitest-only native/WASM checks (no @types/node in the web build).
declare module 'node:fs' {
  export function readFileSync(path: URL): Uint8Array<ArrayBuffer>;
  export function readFileSync(path: string, encoding: 'utf8'): string;
  export function mkdirSync(path: string, options: { recursive: true }): string | undefined;
}

declare module 'node:child_process' {
  export function execFileSync(file: string, args: string[], options: {
    cwd: string;
    encoding: 'utf8';
  }): string;
}
declare module 'node:url' {
  export function fileURLToPath(url: URL): string;
}
