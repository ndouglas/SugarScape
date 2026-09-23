import './style.css';
import { Engine } from './engine';
import { buildDisplay } from './ui/display';
import { h } from './ui/dom';
import { GridView } from './ui/grid-view';
import { RulesPanel } from './ui/rules-panel';
import { Tabs } from './ui/tabs';
import { buildToolbar } from './ui/toolbar';

export function showBanner(message: string, action?: { label: string; run: () => void }): void {
  const banner = document.querySelector<HTMLElement>('#banner')!;
  banner.replaceChildren(
    ...[
      h('span', {}, message),
      action ? h('button', { onclick: action.run }, action.label) : null,
      h('button', { onclick: () => (banner.hidden = true), 'aria-label': 'Dismiss' }, '×'),
    ].filter((child): child is HTMLElement => child !== null),
  );
  banner.hidden = false;
}

async function main(): Promise<void> {
  const engine = await Engine.create();
  const grid = new GridView(document.querySelector<HTMLCanvasElement>('#grid')!, engine);
  document.querySelector('#toolbar')!.append(buildToolbar(engine));
  document.querySelector('#display')!.append(buildDisplay(engine));

  const tabs = new Tabs(document.querySelector('#tabs')!, document.querySelector('#panel-body')!);
  tabs.add('Rules', new RulesPanel(engine).el);

  let dirty = true;
  for (const event of ['reset', 'tick', 'config', 'display', 'select', 'edit'] as const) {
    engine.on(event, () => (dirty = true));
  }
  const loop = () => {
    try {
      if (engine.running) engine.advance();
      if (dirty) {
        engine.trackSelection();
        grid.draw();
        dirty = false;
      }
    } catch (e) {
      // A Rust panic leaves the WASM instance unusable; reloading keeps any #s= share state.
      engine.running = false;
      console.error(e);
      showBanner('The simulation crashed.', { label: 'Reload', run: () => location.reload() });
      return;
    }
    requestAnimationFrame(loop);
  };
  requestAnimationFrame(loop);
}

main().catch((e) => showBanner(`Failed to start: ${e instanceof Error ? e.message : String(e)}`));
