import { h } from '../ui/dom';
import type { Checkpoint, Json } from './types';
import type { RenderDescriptor } from './catalog';
import { obj, list, valueText, fact, details, probability } from './presentation';
function reportNode(title: string, value: Json | undefined): HTMLElement {
  const text = typeof value === 'boolean' ? (value ? 'Positive' : 'Negative') : valueText(value);
  return h('div', { class: 'episode-report-node' }, h('strong', {}, title), h('span', {}, text));
}
/** Only actual checkpoint API stages are displayed. Predictive distributions are not reports. */
export function renderTestimony(checkpoint: Checkpoint, descriptor: RenderDescriptor): HTMLElement {
  const pub = obj(checkpoint.public), localEntries = Object.entries(checkpoint.local);
  const observations = h('section', { class: 'episode-observations' }, h('h3', {}, 'Observed evidence'));
  const inspector = h('aside', { class: 'episode-inspector' }, h('h3', {}, checkpoint.researcher ? 'Researcher and available listener beliefs' : 'Available listener beliefs'));
  if (descriptor.id === 'testimony') {
    const evidence = list(pub.evidence);
    const stream = h('ol', { class: 'episode-evidence-flow' });
    for (const raw of evidence) { const r = obj(raw), c = obj(r.content);
      stream.append(h('li', {}, reportNode(`Record ${r.id}`, c.kind === 'verified' ? `Verified proposition ${c.proposition}: ${valueText(c.value)}` : `Speaker ${c.speaker} reports ${c.positive ? 'positive' : 'negative'} on signal group ${c.group}`)));
    }
    if (!evidence.length) stream.append(h('li', {}, 'Prior: no evidence observed.'));
    observations.append(stream);
    if (pub.error) observations.append(fact('Evidence error', pub.error));
    if (pub.attempted_record) observations.append(fact('Attempted observation', pub.attempted_record));
    for (const [agent, raw] of localEntries) {
      inspector.append(h('h4', {}, `Agent ${agent}`));
      if (!raw) { inspector.append(h('p', {}, 'No delivered belief available.')); continue; }
      const snapshot = obj(obj(raw).snapshot);
      for (const value of list(snapshot.propositions)) { const p = obj(value); inspector.append(probability(`Proposition ${p.id}: probability true`, p.probability_true)); }
      for (const value of list(snapshot.speakers)) { const speaker = obj(value); for (const profile of list(speaker.profiles)) { const p = obj(profile); inspector.append(probability(`Speaker ${speaker.speaker}, profile ${p.id}`, p.probability)); } }
      inspector.append(details('Hypothesis probabilities', snapshot.hypotheses), fact('Evidence count', snapshot.evidence_count));
    }
  } else {
    const observation = obj(pub.observation ?? pub.calibration_view), rules = obj(observation.rules);
    const reporters = list(rules.reporters).length ? list(rules.reporters) : [rules.strategic, rules.fixed];
    const channels = h('div', { class: 'episode-report-channels' });
    for (let i = 0; i < reporters.length; i++) channels.append(h('section', {}, h('h4', {}, `Reporter Agent ${reporters[i]}`), reportNode('Calibration report', list(observation.calibration_reports)[i]), reportNode('Live report', list(observation.live_reports)[i])));
    observations.append(fact('Verified calibration truth', observation.calibration_truth), channels, h('div', { class: 'episode-listener-node' }, h('strong', {}, `Listener Agent ${valueText(rules.decider)}`), h('p', {}, pub.response_label ? String(pub.response_label) : 'Decision from the delivered public view'), fact('Action', pub.action)), h('p', { class: 'hint' }, 'Reports describe what was said. Private signals and live truth are unavailable for this conditional case.'));
    for (const [agent, raw] of localEntries) {
      inspector.append(h('h4', {}, `Agent ${agent}`));
      if (!raw) { inspector.append(h('p', {}, 'No delivered local view available.')); continue; }
      const local = obj(raw), decision = obj(local.decision), calibration = obj(local.calibration_belief);
      if (local.decision) inspector.append(probability('Listener probability of live truth', decision.posterior_true), fact('Action', decision.action));
      else inspector.append(h('p', {}, 'No decision at this API stage.'));
      if (decision.posterior_true === null) inspector.append(h('p', { class: 'hint' }, decision.posterior_availability === 'not_exposed_by_frozen_listener' ? 'The frozen listener API does not expose a posterior.' : 'This listener does not supply a posterior.'));
      const policies = list(decision.policies ?? calibration.policies);
      if (policies.length) { inspector.append(h('h4', {}, 'Policy hypotheses under the disclosed prior')); for (const value of policies) { const p = obj(value); inspector.append(probability(`Policy ${p.canonical_bits}`, p.probability)); } }
      if (calibration.live) inspector.append(details('Predictive distribution of possible live reports', calibration.live), h('p', { class: 'hint' }, 'These are predictions over possibilities, not observed future reports.'));
    }
    observations.append(details('Supplied public rules', rules));
  }
  const assumptions = pub.inference_assumptions;
  observations.append(details('Supplied inference assumptions', assumptions ?? 'The named listener uses its declared model.'));
  if (checkpoint.researcher) inspector.append(details('Researcher reference information', checkpoint.researcher, true));
  return h('div', {}, fact('Stage', checkpoint.kind.replace(/_/g, ' ')), h('div', { class: 'episode-stage' }, observations, inspector));
}
