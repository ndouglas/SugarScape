import { h } from './dom';
import { showNotice } from './notice';

const message = (e: unknown): string => (e instanceof Error ? e.message : String(e));

/**
 * Share: Copy link (the whole session, or a comparison) and Open session… (a session file).
 * A long link is still copied, with a notice (Decision 6).
 */
export function buildShareMenu(opts: {
  link: () => Promise<{ hash: string; notice?: string }>;
  open: (file: File) => Promise<void>;
}): HTMLElement {
  const copy = h(
    'button',
    { title: 'Copy a link that replays this session: the setup, the painted maps and every edit with its tick' },
    'Copy link',
  );
  copy.addEventListener('click', async () => {
    try {
      const { hash, notice } = await opts.link();
      history.replaceState(null, '', hash);
      try {
        await navigator.clipboard.writeText(location.href);
        copy.textContent = 'Link copied';
      } catch {
        copy.textContent = 'Link in address bar';
      }
      if (notice) showNotice(notice, 10_000);
      setTimeout(() => (copy.textContent = 'Copy link'), 2000);
    } catch (e) {
      showNotice(`Could not create the link (${message(e)})`, 10_000);
    }
  });
  const file = h('input', { type: 'file', accept: '.json,application/json', hidden: true, 'aria-label': 'Session file to open' });
  file.addEventListener('change', async () => {
    const chosen = file.files?.[0];
    file.value = '';
    if (chosen) await opts.open(chosen);
  });
  const open = h(
    'button',
    { title: 'Open a session file saved with Export → Session (JSON)', onclick: () => file.click() },
    'Open session…',
  );
  return h('details', { class: 'menu' }, h('summary', {}, 'Share'), h('div', { class: 'menu-items' }, copy, open, file));
}
