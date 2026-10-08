import { h } from '../ui/dom';
import type { Engine } from '../engine';
import type { Json } from '../episodes/types';
import type { Sweep } from './types';
import { ExperimentsView } from './view';
import { EpisodeView } from '../episodes/view';
/** Keeps the legacy sweep view intact while preserving paused episode playback across navigation. */
export class ExperimentsShell {
  readonly el: HTMLElement;
  private readonly sweeps: ExperimentsView;
  private episodes: EpisodeView | null = null;
  private readonly content = h('div');
  private readonly sweepButton = h('button', { 'aria-pressed': true, onclick: () => this.show('sweeps') }, 'Sweeps');
  private readonly episodeButton = h('button', { 'aria-pressed': false, onclick: () => this.show('episodes') }, 'Episodes');
  constructor(engine: Engine) {
    this.sweeps = new ExperimentsView(engine);
    this.content.append(this.sweeps.el);
    this.el = h('div', {}, h('div', { class: 'view-switch experiment-switch', role: 'group', 'aria-label': 'Experiment type' }, this.sweepButton, this.episodeButton), this.content);
  }
  private show(view: 'sweeps' | 'episodes'): void {
    if (view === 'episodes' && !this.episodes) { this.episodes = new EpisodeView(); this.content.append(this.episodes.el); }
    if (view === 'sweeps') this.episodes?.pause();
    this.sweeps.el.hidden = view !== 'sweeps';
    if (this.episodes) this.episodes.el.hidden = view !== 'episodes';
    this.sweepButton.setAttribute('aria-pressed', String(view === 'sweeps'));
    this.episodeButton.setAttribute('aria-pressed', String(view === 'episodes'));
  }
  openSweep(sweep: Sweep): void { this.sweeps.openSweep(sweep); this.show('sweeps'); }
  openInput(input: Json): void { this.show('episodes'); this.episodes!.openInput(input); }
  pause(): void { this.episodes?.pause(); }
  dispose(): void { this.episodes?.dispose(); }
}
