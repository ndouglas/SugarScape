import { h } from '../ui/dom';
import type { Checkpoint, Json } from './types';
import type { RenderDescriptor } from './catalog';
import { obj, list, valueText, fact, details } from './presentation';
import { surfaceDisplay, surfaceSummary, surfaceBelief } from './surface-projection';

function field(symbol: Json, lineage: Json, title: string): HTMLElement {
  return h('section', { class: 'surface-field' }, h('h4', {}, title), h('strong', { class: 'surface-symbol' }, valueText(symbol)), fact('Original write symbol', obj(lineage).symbol), details('Write origin', lineage));
}
export function renderSurface(at: Checkpoint, descriptor: RenderDescriptor): HTMLElement {
  const display = obj(surfaceDisplay(at)), agents = list(display.agents).map(obj), researcher = obj(display.researcher);
  const ids = obj(agents.find(a => a.available)?.ids);
  const names = list(ids.agents);
  const diagram = h('section', { class: 'surface-diagram', 'aria-label': 'Interaction surface' });
  for (const [index, name] of names.entries()) {
    const own = agents.find(a => a.id === name);
    diagram.append(h('div', { class: `surface-agent${own ? ' selected' : ''}` }, h('h3', {}, `Agent ${valueText(name)}`), h('p', {}, `Role ${index === 0 ? 'A' : 'B'}`), fact('Credits remaining', own?.credits), own ? fact('Own private bit', own.privateBit) : h('p', {}, 'Private view unavailable')));
  }
  const surface = h('div', { class: 'surface-object' }, h('h3', {}, valueText(ids.surface ?? 'Interaction surface')));
  if (display.researcher !== null) {
    surface.append(h('p', {}, 'Researcher · physical state'), fact('Actual mechanism', researcher.environment), fact('Ownership', researcher.topology));
    const fields = list(researcher.fields).map(obj);
    if (researcher.topology === 'shared_field' && fields.length) surface.append(field(fields[0].symbol, fields[0].lineage, 'Shared field · visible to A and B'));
    else for (const f of fields) surface.append(field(f.symbol, f.lineage, `Private field · visible to ${valueText(f.visible_to)}`));
  } else {
    surface.append(h('p', {}, 'Opaque surface · ownership unavailable'), h('p', { class: 'surface-symbol' }, 'Current storage unobserved'));
    for (const a of agents) surface.append(fact('Last paid read', obj(a.observation).symbol), fact('Observed at', obj(a.observation).observed_at));
    surface.append(h('p', { class: 'hint' }, 'A dated read is an observation, not a fresh view of storage. An accepted write does not establish peer receipt.'));
  }
  diagram.insertBefore(surface, diagram.children[1] ?? null);
  const inspector = h('section', { class: 'episode-inspector' }, h('h3', {}, 'Knowledge and paid choices'));
  for (const a of agents) {
    const pane = h('section', {}, h('h4', {}, `Agent ${valueText(a.id)}`), fact('Delivered clock', a.delivered), fact('Inference', a.inferenceStatus));
    if (!a.available) pane.append(h('p', {}, 'Local view unavailable at this checkpoint.'));
    else {
      pane.append(details('Current native belief', surfaceBelief(a.belief), true), fact('Current prediction', a.prediction));
      if (a.discovery) pane.append(details('Nominal catalog identification', a.discovery, true), h('p', { class: 'hint' }, 'Supplied or observed catalog certainty does not establish the actual mechanism.'));
      if (a.lastSupportedBelief) pane.append(details('Last supported belief · historical only', surfaceBelief(a.lastSupportedBelief)));
      if (a.decision) {
        const decision = obj(a.decision);
        pane.append(fact('Own choice', decision.choice ?? a.decision));
        if (decision.alternatives) {
          pane.append(h('p', {}, valueText(a.decisionLabel)), fact('Continuation policy', decision.continuation_policy));
          const alternatives = h('ul', { class: 'surface-alternatives' });
          for (const alt of list(decision.alternatives).map(obj)) alternatives.append(h('li', {}, fact(valueText(alt.choice), alt.value)));
          pane.append(alternatives);
        }
      }
    }
    inspector.append(pane);
  }
  const events = h('section', { class: 'surface-events' }, h('h3', {}, 'Delivered event strip'));
  for (const a of agents) {
    const strip = h('ol', { class: 'episode-events' });
    for (const event of list(a.events)) strip.append(h('li', {}, valueText(event)));
    events.append(h('h4', {}, `Agent ${valueText(a.id)} · own history`), strip.childNodes.length ? strip : h('p', {}, 'No delivered events yet.'));
  }
  if (display.researcher !== null) events.append(details('Researcher · current paid event', researcher.event));
  return h('div', { class: 'surface-view' }, h('div', { class: 'surface-clock' }, fact('Public clock', display.clock), fact('Status', display.status), fact('Latest public phase reset', display.reset), descriptor.id === 'active_surface' ? fact('Public probe stop · completed probes', display.probeStop) : null), h('div', { class: 'episode-stage' }, diagram, inspector), events);
}
export function renderSurfaceResult(payload: Json): HTMLElement {
  const summary = surfaceSummary(payload);
  const body = h('div', { class: 'surface-result' }, fact('Completion', summary.status));
  for (const [i, spent] of summary.spent.entries()) body.append(h('section', {}, h('h4', {}, `Agent role ${i === 0 ? 'A' : 'B'}`), fact('Credits spent', spent), fact('Realized reward', summary.reward[i]), fact('Realized net utility', summary.net[i]), fact('Correct predictions', summary.correct[i])));
  if (summary.status !== 'complete') body.append(h('p', {}, 'Unsupported history: the episode stopped at the last charged observation. Terminal reward, net utility, and accuracy are unavailable.'));
  body.append(details('Native final metrics', obj(payload).metrics));
  if (obj(payload).failure) body.append(details('Termination', obj(payload).failure));
  return body;
}
