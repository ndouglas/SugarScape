import { h } from '../ui/dom';
import type { Checkpoint } from './types';
import type { RenderDescriptor } from './catalog';
import { obj, list, valueText, label, fact, details, svgElement } from './presentation';
/** Receives only one permitted checkpoint; there is no full record or input access here. */
export function renderGame(checkpoint: Checkpoint, descriptor: RenderDescriptor): HTMLElement {
  const pub = obj(checkpoint.public);
  const localEntries = Object.entries(checkpoint.local);
  const diagram = svgElement('svg', { viewBox: '0 0 560 340', role: 'img', 'aria-label': `${descriptor.title}: current roster and observed actions`, class: 'episode-game-diagram' });
  const roster = list(pub.roster);
  const positions = new Map<string, { x: number; y: number }>();
  roster.forEach((value, i) => { const angle = i * Math.PI * 2 / roster.length - Math.PI / 2; positions.set(String(obj(value).id), { x: 280 + Math.cos(angle) * 190, y: 160 + Math.sin(angle) * 112 }); });
  const events = h('ul', { class: 'episode-events' });
  const edges = new Set<string>();
  function edge(source: string, target: string, text: string, pending: boolean): void {
    const a = positions.get(source), b = positions.get(target), key = `${source}:${target}:${text}`;
    if (!a || !b || edges.has(key)) return;
    edges.add(key);
    diagram.append(svgElement('line', { x1: a.x, y1: a.y, x2: b.x, y2: b.y, class: pending ? 'episode-edge pending' : 'episode-edge' }), svgElement('text', { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 - 5, class: 'episode-edge-label', 'text-anchor': 'middle' }, text));
  }
  const knowledge = h('aside', { class: 'episode-inspector' }, h('h3', {}, checkpoint.researcher ? 'Researcher — delivered Agent views' : 'Agent knowledge'));
  for (const [agent, raw] of localEntries) {
    const local = obj(raw), request = obj(local.request), observation = obj(request.observation), submission = obj(local.submission), response = obj(submission.response), action = obj(response.action);
    const pane = h('section', {}, h('h4', {}, `Agent ${agent}`));
    if (raw === null) pane.append(h('p', {}, 'No request delivered at this checkpoint.'));
    else {
      pane.append(fact('Request delivered', local.delivered_clock), fact('Objective', observation.objective), fact('Attention capacity', obj(request.legal).attention_capacity), fact('Accusation budget', observation.accusation_budget));
      if (local.submission) {
        pane.append(fact(`${label(String(submission.status))} submission`, action));
        if (action.kind === 'watch') for (const target of list(action.agents)) edge(agent, String(target), `${agent} watches ${target} (${submission.status})`, submission.status === 'pending');
        if (action.target !== undefined) edge(agent, String(action.target), `${agent}: ${action.kind} (${submission.status})`, submission.status === 'pending');
      }
      const observed = list(observation.events);
      pane.append(details('Available observations', observation.events ?? []), details('Granted capabilities and legal actions', { grants: observation.grants ?? null, legal: request.legal ?? null }));
      for (const event of observed) { const content = obj(obj(event).content); events.append(h('li', {}, `Agent ${agent} observed at round ${valueText(obj(event).round)}: ${valueText(content)}`)); if (content.source !== undefined && content.target !== undefined) edge(String(content.source), String(content.target), `${agent} observed ${content.kind}`, false); }
    }
    knowledge.append(pane);
  }
  for (const value of roster) { const r = obj(value), id = String(r.id), at = positions.get(id)!; const selected = id in checkpoint.local;
    const g = svgElement('g', {});
    g.append(svgElement('circle', { cx: at.x, cy: at.y, r: 31, class: `episode-agent ${selected ? 'selected' : ''} ${r.status === 'active' ? '' : 'inactive'}` }), svgElement('text', { x: at.x, y: at.y, 'text-anchor': 'middle' }, `Agent ${id}`), svgElement('text', { x: at.x, y: at.y + 47, 'text-anchor': 'middle', class: 'episode-status-label' }, String(r.status))); diagram.append(g);
  }
  if (!roster.length) diagram.append(svgElement('text', { x: 280, y: 155, 'text-anchor': 'middle' }, 'Terminal roster unavailable'));
  for (const event of list(pub.events)) events.append(h('li', {}, `Public: ${valueText(event)}`));
  if (!events.childNodes.length) events.append(h('li', {}, 'No delivered observations at this checkpoint.'));
  if (checkpoint.researcher) knowledge.append(details('Researcher current request', checkpoint.researcher));
  return h('div', {}, h('div', { class: 'episode-clock' }, fact('Phase and clock', checkpoint.clock), fact('Submission status', pub.submission_status)), h('div', { class: 'episode-stage' }, h('section', {}, diagram, h('p', { class: 'hint' }, 'Dashed edges are pending submissions. Solid edges are committed submissions or labeled observations.'), h('h3', {}, 'Delivered events'), events), knowledge), pub.outcome ? fact('Public outcome', pub.outcome) : null);
}
