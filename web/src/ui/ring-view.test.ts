import { describe, expect, it } from 'vitest';
import type { Engine } from '../engine';
import type { RingState } from '../protocol';
import { RingView } from './ring-view';

/** A canvas whose 2-D context counts the sites and dots filled. */
function fakeCanvas() {
  const ctx = { fills: 0, fillStyle: '', lineWidth: 0, strokeStyle: '' } as Record<string, unknown> & { fills: number };
  for (const op of ['fillRect', 'beginPath', 'arc', 'closePath', 'stroke']) ctx[op] = () => {};
  ctx.fill = () => ctx.fills++;
  const canvas = { width: 0, height: 0, getContext: () => ctx, addEventListener: () => {} };
  return { canvas: canvas as unknown as HTMLCanvasElement, ctx };
}

describe('the ring view', () => {
  it('draws whatever ring state the engine holds now, so a snapshot after the reset fills it', () => {
    // After a switch to Ring World the reset reply has no ring state; the next snapshot brings it.
    const engine = { model: 'ring', ring: null as RingState | null, config: { capacity: 4 }, selection: null };
    const { canvas, ctx } = fakeCanvas();
    const view = new RingView(canvas, engine as unknown as Engine);
    view.draw();
    expect(ctx.fills).toBe(0);
    engine.ring = { sugar: new Float64Array([1, 2, 3, 4, 0]), agents: new Uint32Array([0, 3]) };
    view.draw();
    expect(ctx.fills).toBe(5 + 2);
  });
});
