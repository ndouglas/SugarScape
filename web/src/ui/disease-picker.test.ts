import { describe, expect, it } from 'vitest';
import { DiseaseListPoll, diseaseOptions, PickerWorldGate } from './disease-picker';

const list = [
  { id: 0, bits: '101', carriers: 4 },
  { id: 1, bits: '0011', carriers: 0 },
];

describe('diseaseOptions', () => {
  it('lists diseases by id and bits, with New disease first when allowed', () => {
    expect(diseaseOptions(list, true)).toEqual([
      ['-1', 'New disease'],
      ['0', '#0 · 101'],
      ['1', '#1 · 0011'],
    ]);
  });
  it('omits New disease for vaccination', () => {
    expect(diseaseOptions(list, false)).toEqual([
      ['0', '#0 · 101'],
      ['1', '#1 · 0011'],
    ]);
  });
});

describe('DiseaseListPoll', () => {
  it('asks at once when first opened, then only once the tick moves, at most every minMs', () => {
    const poll = new DiseaseListPoll(250);
    expect(poll.due(0, 5)).toBe(true);
    poll.received(0, 5);
    expect(poll.due(1000, 5)).toBe(false); // paused: nothing changed
    expect(poll.due(100, 6)).toBe(false); // ticked, but too soon
    expect(poll.due(300, 6)).toBe(true);
  });

  it('asks again after an edit without a tick, still no more often than minMs', () => {
    const poll = new DiseaseListPoll(250);
    poll.received(0, 5);
    poll.invalidate();
    expect(poll.due(100, 5)).toBe(false);
    expect(poll.due(300, 5)).toBe(true);
    poll.received(300, 5);
    expect(poll.due(1000, 5)).toBe(false);
  });

  it('asks at once when expedited (the tool reopened, or a reset)', () => {
    const poll = new DiseaseListPoll(250);
    poll.received(0, 5);
    poll.expedite();
    expect(poll.due(1, 5)).toBe(true);
  });
});

describe('PickerWorldGate', () => {
  it('lets presses and drags on the listed world edit', () => {
    const gate = new PickerWorldGate();
    expect(gate.allows('down', true)).toBe(true);
    expect(gate.allows('drag', true)).toBe(true);
  });
  it('makes a press on the other world only a switch, drag included; the next press acts', () => {
    const gate = new PickerWorldGate();
    expect(gate.allows('down', false)).toBe(false);
    // The press switched the picker, so the gesture's drags now report the world as listed.
    expect(gate.allows('drag', true)).toBe(false);
    expect(gate.allows('down', true)).toBe(true);
    expect(gate.allows('drag', true)).toBe(true);
  });
  it('ignores a drag that wanders onto the other world', () => {
    const gate = new PickerWorldGate();
    expect(gate.allows('down', true)).toBe(true);
    expect(gate.allows('drag', false)).toBe(false);
  });
});
