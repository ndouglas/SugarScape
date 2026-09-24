import { h } from './dom';

export class Tabs {
  private entries: { label: string; button: HTMLButtonElement; el: HTMLElement; onShow?: (v: boolean) => void }[] = [];

  constructor(private nav: HTMLElement, private body: HTMLElement) {}

  add(label: string, el: HTMLElement, onShow?: (visible: boolean) => void): void {
    const button = h('button', { role: 'tab', onclick: () => this.show(label) }, label);
    this.nav.append(button);
    this.body.append(el);
    this.entries.push({ label, button, el, onShow });
    if (this.entries.length === 1) this.show(label);
    else el.hidden = true;
  }

  show(label: string): void {
    for (const e of this.entries) {
      const on = e.label === label;
      e.button.setAttribute('aria-selected', String(on));
      e.el.hidden = !on;
      e.onShow?.(on);
    }
  }

  /** Hides or shows a tab's button; hiding the selected tab selects the first visible one. */
  setHidden(label: string, hidden: boolean): void {
    const entry = this.entries.find((e) => e.label === label);
    if (!entry) return;
    entry.button.hidden = hidden;
    if (hidden && entry.button.getAttribute('aria-selected') === 'true') {
      const first = this.entries.find((e) => !e.button.hidden);
      if (first) this.show(first.label);
    }
  }
}
