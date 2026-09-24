import { addGroup, canAddGroup, defaultGroups, moveBoundary, removeGroup, threeTribes } from '../groups';
import type { Config } from '../types';
import { h } from './dom';
import { bind, editor, num, type Commit, type Editor, type Sync } from './goods-editor';

/** Group `k`: color and name apply live; its range ends rebuild the world. */
function groupRow(config: Config, k: number, commit: Commit, syncs: Sync[]): HTMLElement {
  const live = (f: (c: Config) => void) => commit(f, false);
  const name = h('input', { type: 'text', maxLength: 16, class: 'name', 'aria-label': `Group ${k + 1} name` });
  bind(syncs, name, (c) => (name.value = c.culture.groups[k].name));
  name.addEventListener('change', () => live((c) => (c.culture.groups[k].name = name.value)));
  const color = h('input', { type: 'color', 'aria-label': `Group ${k + 1} color` });
  bind(syncs, color, (c) => (color.value = c.culture.groups[k].color));
  color.addEventListener('change', () => live((c) => (c.culture.groups[k].color = color.value)));
  const end = (which: 'min' | 'max') =>
  {
    const input = num(syncs, (c) => c.culture.groups[k].zeros[which], 0, config.tag_length, 1, (v) =>
      commit((c) => moveBoundary(c.culture.groups, k, which, v), true),
    );
    input.setAttribute('aria-label', `Group ${k + 1} ${which === 'min' ? 'fewest' : 'most'} zeros`);
    return input;
  };
  const remove = h('button', {
    disabled: config.culture.groups.length <= 1,
    title: 'Remove this group (its zero counts join a neighbour; rebuilds the world)',
    'aria-label': `Remove group ${k + 1}`,
    onclick: () => commit((c) => removeGroup(c, k), true),
  }, '×');
  return h('div', { class: 'row' }, color, name, h('span', { class: 'hint' }, 'zeros'), end('min'), h('span', { class: 'hint' }, 'to'), end('max'), remove);
}

/** The Culture section's groups table, for `config`'s `groupsEditorSignature`. */
export function groupsEditor(config: Config, commit: Commit): Editor {
  const syncs: Sync[] = [];
  const el = h(
    'div',
    { class: 'tag-groups' },
    h(
      'div',
      { class: 'row' },
      h('button', { onclick: () => commit((c) => (c.culture.groups = defaultGroups(c.tag_length)), true) }, 'Two tribes (book)'),
      h('button', {
        disabled: config.tag_length < 2,
        title: 'Blue, Green and Red: thirds of the zero counts (0–3, 4–7, 8–11 on 11-bit tags)',
        onclick: () => commit((c) => (c.culture.groups = threeTribes(c.tag_length)), true),
      }, 'Three tribes (book)'),
    ),
    ...config.culture.groups.map((_, k) => groupRow(config, k, commit, syncs)),
    h('button', { disabled: !canAddGroup(config), onclick: () => commit((c) => addGroup(c), true) }, 'Add group'),
  );
  return editor(el, syncs, config);
}
