import { describe, expect, it } from 'vitest';
import { nextSpeed, shortcutFor } from './shortcuts';

const key = (k: string, target: { tagName?: string; isContentEditable?: boolean } | null = { tagName: 'BODY' }, mods = {}) => ({
  key: k,
  ctrlKey: false,
  altKey: false,
  metaKey: false,
  target,
  ...mods,
});

describe('shortcutFor', () => {
  it('maps the keys', () => {
    expect(shortcutFor(key(' '))).toBe('play');
    expect(shortcutFor(key('ArrowRight'))).toBe('step');
    expect(shortcutFor(key('ArrowLeft'))).toBe('back');
    expect(shortcutFor(key('['))).toBe('slower');
    expect(shortcutFor(key(']'))).toBe('faster');
    expect(shortcutFor(key('r'))).toBe('reset');
    expect(shortcutFor(key('R'))).toBe('reset');
    expect(shortcutFor(key('?'))).toBe('help');
    expect(shortcutFor(key('x'))).toBeNull();
  });
  it('leaves typing alone', () => {
    for (const tagName of ['INPUT', 'SELECT', 'TEXTAREA']) expect(shortcutFor(key(' ', { tagName }))).toBeNull();
    expect(shortcutFor(key(' ', { tagName: 'DIV', isContentEditable: true }))).toBeNull();
  });
  it('leaves modified keys alone', () => {
    expect(shortcutFor(key('r', undefined, { metaKey: true }))).toBeNull();
    expect(shortcutFor(key('r', undefined, { ctrlKey: true }))).toBeNull();
    expect(shortcutFor(key('ArrowLeft', undefined, { altKey: true }))).toBeNull();
  });
});

describe('nextSpeed', () => {
  const speeds = [1 / 60, 1, 5, 'max'] as const;
  it('walks the list and stops at its ends', () => {
    expect(nextSpeed([...speeds], 1, 1)).toBe(5);
    expect(nextSpeed([...speeds], 1, -1)).toBe(1 / 60);
    expect(nextSpeed([...speeds], 'max', 1)).toBe('max');
    expect(nextSpeed([...speeds], 1 / 60, -1)).toBe(1 / 60);
  });
  it('snaps a speed not in the list to the nearest step in that direction', () => {
    expect(nextSpeed([...speeds], 3, 1)).toBe(5);
    expect(nextSpeed([...speeds], 3, -1)).toBe(1);
  });
});
