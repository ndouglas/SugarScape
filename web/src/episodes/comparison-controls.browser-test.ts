import type { EpisodeView } from './view';

/** Browser behavior regression: run with a real mounted Active/Adaptive episode and a NoProbe pair.
 * Uses native DOM focus/input events. The retained QA runner also exercises actual ArrowRight keys.
 * This is browser-only because the project's Vitest environment is Node (no simulated DOM).
 */
export async function comparisonControlsChecks(view: EpisodeView): Promise<{ name: string; passed: boolean }[]> {
  const checks: { name: string; passed: boolean }[] = [];
  const check = (name: string, passed: boolean) => checks.push({ name, passed });
  const policy = view.el.querySelector<HTMLSelectElement>('[aria-label="Matched policy"]')!;
  const seek = view.el.querySelector<HTMLInputElement>('[aria-label="Comparison checkpoint"]')!;
  const next = [...view.el.querySelectorAll('button')].find(b => b.textContent === 'Next')!;
  const noProbe = [...policy.options].find(o => o.textContent.includes('NoProbe'))!;
  if (!policy || !seek || !noProbe) throw new Error('Requires a mounted Adaptive/NoProbe comparison');
  policy.value = noProbe.value;
  policy.dispatchEvent(new Event('change'));
  const selected = noProbe.value;
  const currentPolicy = () => view.el.querySelector<HTMLSelectElement>('[aria-label="Matched policy"]')!;
  const currentSeek = () => view.el.querySelector<HTMLInputElement>('[aria-label="Comparison checkpoint"]')!;
  next.click();
  check('selected policy survives main Next', currentPolicy().value === selected);
  check('policy control identity survives main Next', currentPolicy() === policy);
  for (const perspective of ['researcher', 'agent:Agent-B', 'agent:Agent-A']) {
    const picker = view.el.querySelector<HTMLSelectElement>('[aria-label="Perspective"]')!;
    picker.value = perspective; picker.dispatchEvent(new Event('change'));
    check(`selected policy survives ${perspective}`, currentPolicy().value === selected);
  }
  for (let step = 1; step <= 3; step++) {
    const slider = currentSeek(); slider.focus();
    slider.value = String(step); slider.dispatchEvent(new Event('input', { bubbles: true }));
    check(`comparison input ${step} retains focused control`, document.activeElement === slider && currentSeek() === slider);
    check(`comparison input ${step} retains its value`, currentSeek().value === String(step));
  }
  policy.focus();
  const initialIndex = view.replay.index;
  view.replay.play();
  const deadline = performance.now() + 3000;
  while (view.replay.index === initialIndex && performance.now() < deadline) await new Promise(resolve => setTimeout(resolve, 20));
  view.pause();
  check('autoplay advanced the native replay', view.replay.index > initialIndex);
  check('selected policy survives autoplay and pause redraw', currentPolicy().value === selected);
  check('policy focus survives autoplay redraw', document.activeElement === policy);
  check('comparison slider identity survives all redraws', currentSeek() === seek);
  const record = view.replay.record!;
  const calibration = record.checkpoints.findIndex(c => JSON.stringify(c.clock).includes('Calibration'));
  view.replay.seek(calibration);
  const join = [...view.el.querySelectorAll('button')].find(b => b.textContent === 'Join current public clock')!;
  join.click();
  check('unvisited public clock remains explicitly unavailable', view.el.querySelector('.episode-comparison')!.textContent!.includes('Unavailable at this public clock'));
  check('unavailable state preserves comparison slider', currentSeek() === seek);
  check('unavailable state preserves chosen policy', currentPolicy().value === selected);
  check('redraw does not create or replace retained records', view.session.records.length === 2 && view.session.shown === record);
  return checks;
}
