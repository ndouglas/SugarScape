import type { WorldName } from '../compare/compare-view';
import { h } from './dom';

interface SlotPanel { el: HTMLElement; setVisible?(visible: boolean): void }

/**
 * A tab's content for one world, or for A and B in Compare (Decision 10): the chosen world's panel
 * shows, under a "Rules for: A | B" switch (`kind: 'switch'`) or a "World A" label (`'label'`,
 * which follows the world last clicked). Outside Compare only A's panel shows, with no header.
 */
export class WorldSlot<P extends SlotPanel> {
  readonly el: HTMLElement;
  private readonly head: HTMLElement;
  private readonly buttons: HTMLButtonElement[] = [];
  private b: P | null = null;
  private which: WorldName = 'A';
  private visible = false;

  constructor(
    readonly a: P,
    private readonly kind: 'switch' | 'label',
    title = '',
  ) {
    if (kind === 'switch') {
      this.buttons = (['A', 'B'] as const).map((w) => h('button', { onclick: () => this.show(w) }, w));
      this.head = h('div', { class: 'world-switch', role: 'group', 'aria-label': title }, h('span', {}, `${title}:`), ...this.buttons);
    } else {
      this.head = h('p', { class: 'world-label' });
    }
    this.head.hidden = true;
    this.el = h('div', { class: 'world-slot' }, this.head, a.el);
  }

  /** Compare's B panel, or null to go back to A alone. */
  setB(b: P | null): void {
    // The old B panel stops asking its world for snapshot extras.
    this.b?.setVisible?.(false);
    this.b?.el.remove();
    this.b = b;
    if (b) this.el.append(b.el);
    this.head.hidden = b === null;
    this.show(b ? this.which : 'A');
  }

  show(which: WorldName): void {
    this.which = this.b ? which : 'A';
    this.a.el.hidden = this.which !== 'A';
    if (this.b) this.b.el.hidden = this.which !== 'B';
    if (this.kind === 'label') this.head.textContent = `World ${this.which}`;
    for (const button of this.buttons) button.setAttribute('aria-pressed', String(button.textContent === this.which));
    this.a.setVisible?.(this.visible && this.which === 'A');
    this.b?.setVisible?.(this.visible && this.which === 'B');
  }

  /** The tab was shown or hidden. */
  setVisible(visible: boolean): void {
    this.visible = visible;
    this.show(this.which);
  }
}
