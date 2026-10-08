import { obj } from './presentation';
import type { EpisodeView } from './view';
import type { Json, StudyDescriptor } from './types';

/** Real DOM/native-worker regressions, run by the retained browser QA harness.
 * Node-only Vitest cannot exercise selects, the actual Run button, or mounted disposal.
 */
export async function openInputChecks(view: EpisodeView, catalog: StudyDescriptor[]): Promise<{ name: string; passed: boolean }[]> {
  const checks: { name: string; passed: boolean }[] = [];
  const check = (name: string, passed: boolean) => checks.push({ name, passed });
  const button = (name: string) => [...view.el.querySelectorAll('button')].find(b => b.textContent === name)!;
  const run = async () => {
    button('Run episode').click();
    const deadline = performance.now() + 60000;
    while (view.el.querySelector<HTMLFieldSetElement>('.episode-setup')!.disabled) {
      if (performance.now() > deadline) throw new Error('Run did not settle');
      await new Promise(resolve => setTimeout(resolve, 20));
    }
  };
  await view.ready;
  for (const descriptor of catalog) {
    const input = structuredClone(descriptor.default_input);
    const prior = view.session.shown;
    view.openInput(input);
    check(`${descriptor.id}: valid opened input stays exact without autorun`, JSON.stringify(view.session.editing) === JSON.stringify(input) && view.session.shown === prior);
    await run();
    check(`${descriptor.id}: default runs and renders`, view.session.shown?.study === descriptor.id && !!view.el.querySelector('.episode-rendered')?.textContent);
    if (descriptor.family === 'surface') continue;
    for (const key of ['policy', 'mode', 'fixture', 'environment', 'listener', 'controller', 'witness', 'catalog']) {
      if (!(key in obj(descriptor.controls))) continue;
      for (const variant of ['unknown', 'missing']) {
        const opened = structuredClone(obj(input));
        if (variant === 'missing') delete opened[key]; else opened[key] = 'not-a-selection';
        const successful = view.session.shown;
        const records = [...view.session.records];
        view.openInput(opened);
        check(`${descriptor.id}/${key}/${variant}: opened input preserved`, JSON.stringify(view.session.editing) === JSON.stringify(opened));
        const control = view.el.querySelector<HTMLSelectElement>(`[aria-label="${key}"]`)!;
        check(`${descriptor.id}/${key}/${variant}: opened selection visibly invalid`, control.selectedOptions[0]?.textContent?.includes('validation on run') === true);
        check(`${descriptor.id}/${key}/${variant}: opening does not run`, view.session.shown === successful);
        await run();
        check(`${descriptor.id}/${key}/${variant}: rejection retains successful records`, view.session.shown === successful && view.replay.record === successful && records.every((record, index) => view.session.records[index] === record));
        const status = view.el.querySelector('[role="status"]')!.textContent!;
        check(`${descriptor.id}/${key}/${variant}: contextual error`, /^(input|policy|mode|fixture|environment|listener|controller|witness|catalog|name|id): .+/.test(status));
      }
    }
  }
  const strategic = catalog.find(d => d.id === 'strategic_reporting')!;
  view.openInput(strategic.default_input);
  const environment = view.el.querySelector<HTMLSelectElement>('[aria-label="environment"]')!;
  const listener = view.el.querySelector<HTMLSelectElement>('[aria-label="listener"]')!;
  // Exercise the real dependent choices: explicit user changes may pick a compatible listener.
  const rows = obj(strategic.controls).listener;
  const byEnvironment = obj(rows).by_environment as { environment: string; values: Json[] }[];
  const different = byEnvironment.find(row => !row.values.includes(listener.value))!;
  environment.value = different.environment;
  environment.dispatchEvent(new Event('change'));
  check('explicit environment change chooses a compatible listener', different.values.includes(obj(view.session.editing).listener));
  const updatedListener = view.el.querySelector<HTMLSelectElement>('[aria-label="listener"]')!;
  updatedListener.value = String(different.values.at(-1));
  updatedListener.dispatchEvent(new Event('change'));
  check('explicit listener selection becomes next input', obj(view.session.editing).listener === different.values.at(-1));
  // Re-rendering another field must not normalize an unrelated malformed opened selector.
  view.openInput({ ...obj(strategic.default_input), policy: 'not-a-selection' });
  const environmentAgain = view.el.querySelector<HTMLSelectElement>('[aria-label="environment"]')!;
  environmentAgain.value = different.environment;
  environmentAgain.dispatchEvent(new Event('change'));
  check('environment edit preserves unrelated malformed policy', obj(view.session.editing).policy === 'not-a-selection');
  const exact = { study: 'wink', seed: '18446744073709551615', policy: 'random', mode: 'diagnostic' };
  view.openInput(exact);
  check('valid nondefault maximum-u64 opened input stays exact', JSON.stringify(view.session.editing) === JSON.stringify(exact));
  return checks;
}

export async function disposeChecks(view: EpisodeView, active: StudyDescriptor): Promise<{ name: string; passed: boolean }[]> {
  const checks: { name: string; passed: boolean }[] = [];
  const check = (name: string, passed: boolean) => checks.push({ name, passed });
  const button = (name: string) => [...view.el.querySelectorAll('button')].find(b => b.textContent === name)!;
  const idle = async () => {
    const deadline = performance.now() + 60000;
    while (view.el.querySelector<HTMLFieldSetElement>('.episode-setup')!.disabled) {
      if (performance.now() > deadline) throw new Error('Run did not settle');
      await new Promise(resolve => setTimeout(resolve, 20));
    }
  };
  view.openInput(active.default_input); button('Run episode').click(); await idle();
  button('Run matched comparison').click(); await idle();
  check('disposal fixture contains actual comparison records', view.session.records.length === 2);
  const shown = view.session.shown;
  view.replay.seek(1); view.replay.play(); view.pause();
  check('pause retains record and checkpoint', view.replay.record === shown && view.replay.index === 1 && !view.replay.playing);
  view.dispose();
  check('dispose releases replay record and resets index', view.replay.record === null && view.replay.index === 0 && !view.replay.playing);
  check('dispose releases session records', view.session.shown === null && view.session.records.length === 0);
  check('dispose clears detached result and comparison content', view.el.querySelector('.episode-rendered')!.childElementCount === 0 && !view.el.querySelector('.episode-comparison')!.textContent!.includes('Independent timelines:'));
  return checks;
}
