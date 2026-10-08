import { h } from '../ui/dom';
import { downloadText } from '../downloads';
import { createEpisodeClient } from './client';
import { EpisodeSession, readEpisodeFile } from './file';
import { EpisodeControls } from './controls';
import { rendererAvailable, renderDescriptor } from './catalog';
import { ReplayController } from './replay';
import { projectCheckpoint, resultPayload, type Perspective } from './projection';
import { renderGame } from './game-view';
import { renderTestimony } from './testimony-view';
import { renderRecorded } from './recorded';
import { encodeEpisode, checkedInput } from './share';
import { obj, details, fact, label } from './presentation';
import type { EpisodeClient, EpisodeRecord, Json, StudyDescriptor } from './types';

export class EpisodeView {
  readonly el: HTMLElement;
  readonly replay: ReplayController;
  readonly session: EpisodeSession;
  readonly ready: Promise<void>;
  private catalog: StudyDescriptor[] = [];
  private editor: EpisodeControls | null = null;
  private perspective: Perspective = { kind: 'agent', agent: '0' };
  private busy = false;
  private disposed = false;
  private pendingInput: Json | undefined;
  private readonly picker = h('select', { 'aria-label': 'Episode study', onchange: () => this.pick() });
  private readonly editorSlot = h('div');
  private readonly setup = h('fieldset', { class: 'episode-setup', disabled: true });
  private readonly status = h('p', { role: 'status', class: 'hint' }, 'Loading study catalog…');
  private readonly runButton = h('button', { class: 'primary', onclick: () => void this.run() }, 'Run episode');
  private readonly cancelButton = h('button', { disabled: true, onclick: () => this.client.cancel() }, 'Cancel');
  private readonly stage = h('section', { class: 'episode-rendered' });
  private readonly shownTitle = h('h2', {}, 'Episode');
  private readonly shownNote = h('p', { class: 'hint' }, 'Choose a study and run an episode.');
  private readonly playback = h('fieldset', { class: 'episode-playback', disabled: true });
  private readonly perspectivePicker = h('select', { 'aria-label': 'Perspective', onchange: () => this.changePerspective() });
  private readonly seek = h('input', { type: 'range', min: 0, max: 0, value: 0, 'aria-label': 'Checkpoint', oninput: () => this.replay.seek(Number(this.seek.value)) });
  private readonly counter = h('output', { 'aria-live': 'polite' });
  private readonly playButton = h('button', { onclick: () => { if (this.replay.playing) { this.replay.pause(); this.draw(); } else this.replay.play(); } }, 'Play');
  private readonly retained = h('select', { 'aria-label': 'Retained episode', onchange: () => this.showRecord(this.session.records[Number(this.retained.value)]) });
  private readonly recordedSlot = h('div');
  private readonly recordedButton = h('button', { onclick: () => void this.loadRecorded() }, 'Load recorded study results');
  private readonly fileInput = h('input', { type: 'file', accept: '.json,application/json', hidden: true, onchange: () => void this.openFile() });
  constructor(private client: EpisodeClient = createEpisodeClient()) {
    this.session = new EpisodeSession(client);
    this.replay = new ReplayController(() => this.draw());
    this.setup.append(h('div', { class: 'row' }, h('label', {}, 'Study ', this.picker), h('button', { onclick: () => this.fileInput.click() }, 'Open episode…'), this.fileInput), this.editorSlot, h('div', { class: 'row' }, this.runButton, h('button', { onclick: () => void this.share() }, 'Share input link'), h('button', { onclick: () => void this.export() }, 'Export shown episode')));
    const previous = h('button', { onclick: () => this.replay.step(-1) }, 'Previous');
    const next = h('button', { onclick: () => this.replay.step() }, 'Next');
    this.playback.append(h('div', { class: 'row' }, h('button', { onclick: () => this.replay.reset() }, 'Reset'), previous, this.playButton, next, h('label', {}, 'Perspective ', this.perspectivePicker)), h('div', { class: 'episode-timeline' }, this.seek, this.counter));
    this.el = h('div', { class: 'episode-view' }, this.setup, h('div', { class: 'row' }, this.cancelButton, this.status), h('div', { class: 'episode-shown-heading' }, this.shownTitle, this.retained), this.shownNote, this.playback, this.stage, this.recordedButton, this.recordedSlot);
    this.ready = this.initialize();
  }
  private async initialize(): Promise<void> {
    this.busy = true;
    try {
      this.catalog = await this.client.catalog();
      if (this.disposed) return;
      this.picker.replaceChildren(...this.catalog.map(d => h('option', { value: d.id }, `${d.title}${rendererAvailable(d) ? '' : ' (visualization unavailable)'}`)));
      this.pick(); this.status.textContent = '';
      if (this.pendingInput !== undefined) { const input = this.pendingInput; this.pendingInput = undefined; this.openInput(input); }
    } catch (error) { this.error(error); }
    finally { this.setBusy(false); }
  }
  openInput(input: Json): void {
    checkedInput(input);
    if (!this.catalog.length) { this.pendingInput = structuredClone(input); return; }
    const descriptor = this.catalog.find(d => d.id === obj(input).study);
    if (!descriptor) throw new Error('unknown episode study');
    this.picker.value = descriptor.id;
    this.setEditor(descriptor, input);
    this.status.textContent = 'Input opened for editing. Run episode to evaluate it.';
  }
  private pick(): void { const d = this.catalog.find(d => d.id === this.picker.value); if (d) this.setEditor(d, d.default_input); }
  private setEditor(descriptor: StudyDescriptor, input: Json): void {
    this.editor = new EpisodeControls(descriptor, input, edited => this.session.edit(edited));
    this.session.edit(this.editor.input());
    this.editorSlot.replaceChildren(h('p', {}, descriptor.question), h('p', { class: 'hint' }, descriptor.supplied), this.editor.el);
    this.runButton.disabled = !rendererAvailable(descriptor);
  }
  private setBusy(on: boolean): void { this.busy = on; this.setup.disabled = on || !this.catalog.length || this.disposed; this.cancelButton.disabled = !on; this.recordedButton.disabled = on || this.disposed; }
  private error(error: unknown): void { if (!this.disposed) this.status.textContent = error instanceof Error ? error.message : String(error); }
  private async run(): Promise<void> {
    if (this.busy || !this.editor) return;
    this.setBusy(true); this.status.textContent = 'Running episode…';
    try { const record = await this.session.run(this.editor.input()); if (!this.disposed) { this.showRecord(record); this.status.textContent = 'Episode complete. Playback uses its recorded checkpoints.'; } }
    catch (error) { this.error(error); } finally { this.setBusy(false); }
  }
  private async openFile(): Promise<void> {
    const file = this.fileInput.files?.[0]; this.fileInput.value = '';
    if (!file || this.busy) return;
    this.setBusy(true); this.status.textContent = 'Validating imported episode…';
    try { const record = await this.session.import(await readEpisodeFile(file)); if (!this.disposed) { this.showRecord(record); this.status.textContent = `Validated ${file.name}`; } }
    catch (error) { this.error(error); } finally { this.setBusy(false); }
  }
  private async share(): Promise<void> {
    if (!this.editor) return;
    try { const token = await encodeEpisode(this.editor.input()); await navigator.clipboard.writeText(`${location.origin}${location.pathname}${location.search}#e=${token}`); this.status.textContent = 'Input link copied.'; } catch (error) { this.error(error); }
  }
  private async export(): Promise<void> {
    if (this.busy || !this.session.shown) return;
    const study = this.session.shown.study;
    this.setBusy(true);
    try { const text = await this.session.export(); if (!this.disposed) downloadText(`${study}-episode.json`, text, 'application/json'); }
    catch (error) { this.error(error); } finally { this.setBusy(false); }
  }
  private async loadRecorded(): Promise<void> {
    if (this.busy) return;
    this.setBusy(true); this.status.textContent = 'Loading retained measurements…';
    try { const recorded = await this.client.recorded(); if (!this.disposed) { this.recordedSlot.replaceChildren(renderRecorded(recorded)); this.recordedButton.hidden = true; this.status.textContent = 'Retained measurements loaded from their original source.'; } }
    catch (error) { this.error(error); } finally { this.setBusy(false); }
  }
  private showRecord(record: EpisodeRecord): void {
    this.session.shown = record;
    // Do not inspect future local maps merely to populate this checkpoint's perspective menu.
    this.perspective = { kind: 'agent', agent: Object.keys(record.checkpoints[0].local)[0] ?? '0' };
    this.retained.replaceChildren(...this.session.records.map((r, i) => h('option', { value: i, selected: r === record }, `${i + 1}: ${this.catalog.find(d => d.id === r.study)?.title ?? r.study}`)));
    this.replay.load(record);
  }
  private changePerspective(): void { this.perspective = this.perspectivePicker.value === 'researcher' ? { kind: 'researcher' } : { kind: 'agent', agent: this.perspectivePicker.value.slice(6) }; this.draw(); }
  private draw(): void {
    const record = this.replay.record;
    if (!record || this.disposed) return;
    const descriptor = this.catalog.find(d => d.id === record.study);
    if (!descriptor) return;
    const checkpoint = record.checkpoints[this.replay.index];
    const agents = new Set(Object.keys(checkpoint.local));
    for (const row of Array.isArray(obj(checkpoint.public).roster) ? obj(checkpoint.public).roster as Json[] : []) agents.add(String(obj(row).id));
    if (this.perspective.kind === 'agent') agents.add(this.perspective.agent);
    const current = this.perspective.kind === 'researcher' ? 'researcher' : `agent:${this.perspective.agent}`;
    this.perspectivePicker.replaceChildren(...[...agents].map(agent => h('option', { value: `agent:${agent}`, selected: current === `agent:${agent}` }, `Agent ${agent}`)), h('option', { value: 'researcher', selected: current === 'researcher' }, 'Researcher (privileged)'));
    this.playback.disabled = false;
    this.seek.max = String(record.checkpoints.length - 1); this.seek.value = String(this.replay.index);
    this.counter.textContent = `${this.replay.index + 1} / ${record.checkpoints.length} — ${label(checkpoint.kind)}`;
    this.playButton.textContent = this.replay.playing ? 'Pause' : 'Play';
    this.playButton.disabled = matchMedia('(prefers-reduced-motion: reduce)').matches;
    this.playButton.title = this.playButton.disabled ? 'Reduced motion: use Previous, Next, or the checkpoint slider.' : 'Play retained checkpoints';
    this.shownTitle.textContent = descriptor.title;
    this.shownNote.textContent = `Shown successful episode · ${label(record.semantics)}. Editing controls above changes the next run.`;
    const projected = projectCheckpoint(record, this.replay.index, this.perspective);
    const rendered = descriptor.family === 'game' ? renderGame(projected, renderDescriptor(descriptor)) : descriptor.family === 'testimony' ? renderTestimony(projected, renderDescriptor(descriptor)) : h('p', {}, 'Surface visualization unavailable in this intermediate build.');
    const payload = resultPayload(record, this.replay.index, this.perspective);
    const result = h('section', { class: 'episode-result' });
    if (payload !== null) {
      const p = obj(payload);
      result.append(h('div', {}, h('h3', {}, this.perspective.kind === 'researcher' ? 'Researcher — complete episode result' : 'Final episode result'), fact('Availability', p.availability ?? p.status ?? 'complete'), p.expected_payoff !== undefined ? fact('Conditional expected payoff', p.expected_payoff) : null, p.conditional_regret !== undefined ? fact('Conditional expected regret', p.conditional_regret) : null, p.outcome ? fact('Outcome', p.outcome) : null, p.fingerprint ? fact('Fingerprint', p.fingerprint) : null, details('Shown input and rules identity', { input: record.input, rules_identity: record.rules_identity }), p.retained_audit_summary ? details('Retained population and guarantee measurements', p.retained_audit_summary) : null));
      if (record.semantics === 'conditional_case') result.append(h('p', { class: 'hint' }, 'Conditional expectations are not a realized reward. Private truth and signals are unavailable.'));
    } else result.append(h('p', { class: 'hint' }, 'Complete results become available at the final checkpoint or in Researcher perspective.'));
    this.stage.replaceChildren(rendered, result);
  }
  pause(): void { this.replay.pause(); this.draw(); }
  dispose(): void { this.disposed = true; this.replay.pause(); this.session.dispose(); this.recordedSlot.replaceChildren(); }
}
