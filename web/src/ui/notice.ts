import { h } from './dom';

let timer: ReturnType<typeof setTimeout> | undefined;

/** Shows a short, non-error message in the status strip (#notice) for `ms` milliseconds (Decision 7). */
export function showNotice(message: string, ms = 5000): void {
  const el = document.querySelector<HTMLElement>('#notice');
  if (!el) return;
  el.replaceChildren(
    h('span', {}, message),
    h('button', { class: 'link', 'aria-label': 'Dismiss', onclick: () => (el.hidden = true) }, '×'),
  );
  el.hidden = false;
  clearTimeout(timer);
  timer = setTimeout(() => (el.hidden = true), ms);
}
