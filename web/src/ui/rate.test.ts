import { describe, expect, it } from 'vitest';
import { RateMeter } from './rate';

describe('RateMeter', () => {
  it('measures ticks per second over the last window', () => {
    const m = new RateMeter(1000);
    expect(m.rate()).toBeNull();
    m.sample(0, 0);
    m.sample(250, 15);
    m.sample(500, 30);
    expect(m.rate()).toBeCloseTo(60);
    m.sample(1500, 40); // older samples fall out of the window
    expect(m.rate()).toBeCloseTo(10);
  });
  it('starts over when the tick goes back', () => {
    const m = new RateMeter();
    m.sample(0, 100);
    m.sample(250, 200);
    m.sample(500, 5);
    expect(m.rate()).toBeNull();
  });
});
