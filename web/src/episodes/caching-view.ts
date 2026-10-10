import { h } from '../ui/dom';
import type { Checkpoint } from './types';
import type { RenderDescriptor } from './catalog';
import { obj,list,valueText,fact,details } from './presentation';
import { cachingDisplay } from './caching-projection';
import { renderSpatialMap } from './spatial-view';
/** Receives no input, payload, other checkpoint or unsafe descriptor. */
export function renderCaching(at:Checkpoint,_descriptor:RenderDescriptor):HTMLElement {
  const display=obj(cachingDisplay(at)),{map,inspector}=renderSpatialMap(display);
  map.append(h('p',{class:'hint'},'Cache memory is retained evidence, not current physical stock. Evidence tick and age refer to the displayed completed-step boundary. Own actions belong to the recorded interval; no separate action tick is inferred.'));
  for(const a of list(display.agents).map(obj)){const local=obj(a.local),state=obj(a.state),pane=h('section',{},h('h4',{},`Agent ${valueText(a.id)}`),fact('Captured position',state.pos),fact('Holdings',state.holdings),details('Own buried caches',state.caches),details('Remembered cache evidence (original tick and age)',local.seen),details('Own committed actions in recorded interval',local.own_actions),details('Local availability',local.availability));
    if('protection' in local)pane.append(details('Own protection / perceived exposure memory',local.protection),h('p',{class:'hint'},'Expired marks retained memory eligibility at this boundary (age > exposure span). It does not explain a prior action. Actual watchers remain Researcher diagnostics.'));
    if('sender' in local)pane.append(details('Own supplied sender state',local.sender),details('Received public observations (original pre-step tick)',local.received_observations),h('p',{class:'hint'},'A clear visible transfer, including zero, is public evidence. Ambiguous cues do not establish hidden transfer or current stock. Gestures are supplied experimental behavior.'));
    inspector.append(pane);
  }
  if(at.researcher!==null){const research=obj(at.researcher);inspector.append(details('Researcher surface resources',obj(research.display).stocks),h('p',{class:'hint'},'Surface resources are native site.resource[0]. Buried caches belong to each role. Observation actual_stock is a separate actor-cache diagnostic; these are distinct stock namespaces.'),details('Researcher native frame: actual watchers, transfer, actor-cache stock and choices',research.frame));}
  if(obj(at.public).lab_kind==='protection')map.append(h('p',{class:'hint'},'Physical surface-resource stock is unavailable in this native P3 capture. No stock layer is inferred.'));
  return h('div',{class:'spatial-view caching-view'},h('div',{class:'surface-clock'},fact('Native completed-step clock / action interval',at.clock),fact('Native stage',at.kind)),h('div',{class:'spatial-layout'},map,inspector));
}
