import { h } from '../ui/dom';
import { obj, list, label, valueText } from './presentation';
import type { Json, StudyDescriptor } from './types';
/** These are the native family's named selectors, not a user-authored engine configuration. */
export class EpisodeControls {
  readonly el = h('div', { class: 'episode-controls' });
  private draft: Record<string, Json>;
  constructor(private descriptor: StudyDescriptor, input: Json = descriptor.default_input, private changed: (input: Json) => void = () => {}) {
    this.draft = structuredClone(obj(input)); this.render();
  }
  input(): Json { return structuredClone(this.draft); }
  private render(): void {
    this.el.replaceChildren();
    if (this.descriptor.family === 'surface') { this.el.append(h('p', { class: 'hint' }, 'Surface visualization unavailable in this intermediate build.')); return; }
    const controls = obj(this.descriptor.controls);
    const keys = this.descriptor.id === 'wink' ? ['seed', 'policy', 'mode'] : this.descriptor.id === 'testimony' ? ['fixture'] : ['environment', 'listener', 'policy', 'controller', 'witness', 'catalog', 'history'];
    for (const key of keys) {
      if (!(key in controls)) continue;
      const spec = obj(controls[key]);
      let input: HTMLSelectElement | HTMLInputElement;
      if (key === 'history') {
        input = h('input', { type: 'number', min: spec.min, max: spec.max, step: 1, value: this.draft[key], 'aria-label': 'Public history' });
      } else if (key === 'seed') {
        input = h('input', { type: 'text', inputMode: 'numeric', pattern: '[0-9]+', value: this.draft[key], 'aria-label': 'Seed (decimal u64)' });
      } else {
        const perEnvironment = list(spec.by_environment).find(row => obj(row).environment === this.draft.environment);
        const values = list(perEnvironment ? obj(perEnvironment).values : spec.values);
        if (!values.includes(this.draft[key])) this.draft[key] = values[0] ?? '';
        input = h('select', { 'aria-label': label(key) }, ...values.map(value => h('option', { value, selected: value === this.draft[key] }, obj(spec.labels)[String(value)] ? valueText(obj(spec.labels)[String(value)]) : label(String(value)))));
      }
      input.addEventListener('change', () => {
        this.draft[key] = key === 'history' ? Number(input.value) : input.value;
        if (key === 'environment') this.render();
        this.changed(this.input());
      });
      this.el.append(h('label', {}, key === 'history' ? 'Public history' : label(key), input));
    }
    if ('history' in controls) this.el.append(h('p', { class: 'hint' }, 'A conditional public history (0–31). This selects a case; private truth is not sampled.'));
  }
}
