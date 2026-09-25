import type { RunControls } from '../engine';
import { fieldErrorsMessage } from '../errors';
import type { FieldError } from '../types';
import { h } from './dom';

/** One seek at a time; while one is in flight, only the newest request waits to follow it (a fast drag). */
export class SeekQueue {
  private busy: Promise<void> | null = null;
  private next: number | null = null;

  constructor(
    private readonly seek: (tick: number) => Promise<unknown>,
    private readonly onError: (e: unknown) => void = () => {},
  ) {}

  request(tick: number): void {
    this.next = tick;
    if (!this.busy) this.busy = this.drain();
  }

  /** Resolves once every request so far has been sent and answered. */
  async settled(): Promise<void> {
    while (this.busy) await this.busy;
  }

  private async drain(): Promise<void> {
    while (this.next !== null) {
      const t = this.next;
      this.next = null;
      try {
        await this.seek(t);
      } catch (e) {
        this.onError(e);
      }
    }
    this.busy = null;
  }
}

/** ⟲1 and a slider over the ticks this world's branch has reached (0 … reached). */
export class Timeline {
  readonly el: HTMLElement;
  private controls: RunControls | null = null;
  private readonly backButton: HTMLButtonElement;
  private readonly slider: HTMLInputElement;
  private readonly queue: SeekQueue;
  private dragging = false;
  held = false;

  constructor(onError: (e: unknown) => void) {
    // Engine.seek resolves to FieldError[] | null instead of throwing; turn a refusal into a
    // thrown Error so the queue's onError (and its notice) sees it the same way it sees
    // Lockstep.seek's throw.
    this.queue = new SeekQueue(async (t) => {
      const errors = (await this.controls!.seek(t)) as FieldError[] | null;
      if (errors) throw new Error(fieldErrorsMessage(errors));
    }, onError);
    this.backButton = h(
      'button',
      { title: 'Back one tick (←)', 'aria-label': 'Back one tick', onclick: () => this.back() },
      '⟲1',
    );
    this.slider = h('input', { type: 'range', min: 0, max: 0, step: 1, class: 'timeline', 'aria-label': 'Tick' });
    this.slider.addEventListener('pointerdown', () => (this.dragging = true));
    this.slider.addEventListener('pointerup', () => (this.dragging = false));
    this.slider.addEventListener('input', () => {
      if (this.controls?.running) this.controls.setRunning(false);
      this.queue.request(Number(this.slider.value));
    });
    this.el = h('div', { class: 'group timeline-group' }, this.backButton, this.slider);
  }

  bind(controls: RunControls): void {
    this.controls = controls;
    this.sync();
  }

  back(): void {
    const c = this.controls;
    if (!c || this.held || !c.seekable || c.tick === 0) return;
    if (c.running) c.setRunning(false);
    this.queue.request(c.tick - 1);
  }

  sync(): void {
    const c = this.controls;
    if (!c) return;
    const why = c.seekable ? '' : 'This session’s log is full, so it can no longer be rebuilt exactly';
    this.slider.max = String(c.reached);
    // Leave the thumb where the user is dragging it.
    if (!this.dragging) this.slider.value = String(c.tick);
    this.slider.disabled = this.held || !c.seekable;
    this.slider.title = why || `Tick ${c.tick} of ${c.reached} — drag to go back and forth`;
    this.backButton.disabled = this.held || !c.seekable || c.tick === 0;
    if (why) this.backButton.title = why;
  }
}
