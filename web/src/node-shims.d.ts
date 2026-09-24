// The one Node API the Vitest-only WASM test uses (the web build has no @types/node).
declare module 'node:fs' {
  export function readFileSync(path: URL): Uint8Array<ArrayBuffer>;
}
