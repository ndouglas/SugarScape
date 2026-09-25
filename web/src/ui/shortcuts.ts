import type { Speed } from '../engine';
import { h } from './dom';
import type { Toolbar } from './toolbar';

export type Shortcut = 'play' | 'step' | 'back' | 'slower' | 'faster' | 'reset' | 'help';

const KEYS: Record<string, Shortcut> = {
  ' ': 'play',
  ArrowRight: 'step',
  ArrowLeft: 'back',
  '[': 'slower',
  ']': 'faster',
  r: 'reset',
  R: 'reset',
  '?': 'help',
};

const HELP: [string, string][] = [
  ['Space', 'Play / Pause'],
  ['→', 'Step one tick'],
  ['←', 'Back one tick'],
  ['[ / ]', 'Slower / faster'],
  ['R', 'Reset'],
  ['?', 'Show / hide this card'],
];

const TYPING = new Set(['INPUT', 'SELECT', 'TEXTAREA']);

/**
 * The shortcut a key press means, or null (typing in a field, a modifier held, another key, or
 * an element that already handled the key — e.g. an SVG node's own Space/Enter activation).
 */
export function shortcutFor(e: {
  key: string;
  ctrlKey: boolean;
  altKey: boolean;
  metaKey: boolean;
  target: { tagName?: string; isContentEditable?: boolean } | null;
  defaultPrevented?: boolean;
}): Shortcut | null {
  if (e.ctrlKey || e.altKey || e.metaKey) return null;
  if (e.defaultPrevented) return null;
  const t = e.target;
  if (t && (TYPING.has(t.tagName ?? '') || t.isContentEditable)) return null;
  return KEYS[e.key] ?? null;
}

const rank = (s: Speed): number => (s === 'max' ? Infinity : s);

/** The speed one step slower (−1) or faster (1) than `current` in `speeds` (ascending), stopping at the ends. */
export function nextSpeed(speeds: Speed[], current: Speed, dir: -1 | 1): Speed {
  const r = rank(current);
  if (dir === 1) return speeds.find((s) => rank(s) > r) ?? speeds[speeds.length - 1];
  return [...speeds].reverse().find((s) => rank(s) < r) ?? speeds[0];
}

/** Listens for the shortcuts on the document; returns the removal. */
export function installShortcuts(toolbar: Toolbar): () => void {
  const card = h(
    'div',
    { class: 'shortcuts-card', role: 'dialog', 'aria-label': 'Keyboard shortcuts', hidden: true },
    h('h2', {}, 'Keyboard shortcuts'),
    h('dl', {}, ...HELP.flatMap(([k, what]) => [h('dt', {}, h('kbd', {}, k)), h('dd', {}, what)])),
  );
  document.body.append(card);
  const onKey = (e: KeyboardEvent): void => {
    if (e.key === 'Escape' && !card.hidden) {
      card.hidden = true;
      return;
    }
    // An element that already handled the key (e.g. a credit node's Space/Enter select) calls
    // preventDefault() but not stopPropagation(); the bubbled event must not also fire a shortcut.
    if (e.defaultPrevented) return;
    const s = shortcutFor({
      key: e.key,
      ctrlKey: e.ctrlKey,
      altKey: e.altKey,
      metaKey: e.metaKey,
      target: e.target as HTMLElement | null,
      defaultPrevented: e.defaultPrevented,
    });
    if (!s) return;
    e.preventDefault();
    if (s === 'help') card.hidden = !card.hidden;
    else toolbar.shortcut(s);
  };
  document.addEventListener('keydown', onKey);
  return () => {
    document.removeEventListener('keydown', onKey);
    card.remove();
  };
}
