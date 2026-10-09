import { h } from '../ui/dom';
import type { Checkpoint, Json } from './types';
import type { RenderDescriptor } from './catalog';
import { obj,list,valueText,fact,details } from './presentation';
import { spatialDisplay } from './spatial-projection';
const glyphs:Record<string,string>={Unknown:'?',KnownOpen:'·',KnownSolid:'#',Open:'·',Solid:'#','#':'#','.':'·',E:'E',o:'o',w:'w',W:'W'};
function terrain(kind:Json):string {return typeof kind==='string'?kind:'KnownSolid';}
/** The only renderer inputs are permitted checkpoint data and a safe descriptor. */
export function renderSpatial(at:Checkpoint,_descriptor:RenderDescriptor):HTMLElement {
  const display=obj(spatialDisplay(at)),agents=list(display.agents).map(obj),cells=list(display.cells).map(obj);
  const selected=h('section',{class:'spatial-selection','aria-live':'polite'},h('h4',{},'Selected cell'),h('p',{},'Select a cell to inspect its permitted classification and occupants.'));
  const map=h('section',{class:'spatial-map'},h('h3',{},`${valueText(display.mapKind)} map`));
  const inspector=h('section',{class:'episode-inspector spatial-inspector'},h('h3',{},display.researcher===null?'Captured own state':'Current physical state'),fact('Local state availability',display.availability),selected);
  if(cells.length){
    const grid=h('div',{class:'spatial-grid','aria-label':'Current sampled map',style:`--columns:${display.width};--rows:${display.height}`,'data-zoom':'1'});
    const viewport=h('div',{class:'spatial-viewport',tabIndex:0,'aria-label':'Scrollable sampled map'},grid);
    let zoom=1, fittedWidth:number|null=null;
    const setZoom=(n:number)=>{
      zoom=Math.max(.5,Math.min(4,n));grid.dataset.zoom=String(zoom);
      grid.style.width=fittedWidth===null?`${zoom*100}%`:`${fittedWidth*zoom}px`;
      grid.style.minWidth=fittedWidth===null?'':'0';
      if(fittedWidth===null)grid.style.removeProperty('--glyph-size');
      else grid.style.setProperty('--glyph-size',`${Math.min(18,fittedWidth*zoom/Number(display.width)*.75)}px`);
    };
    const fit=()=>{
      zoom=1;grid.dataset.zoom='1';
      const style=getComputedStyle(viewport);
      const availableWidth=viewport.clientWidth-parseFloat(style.paddingLeft)-parseFloat(style.paddingRight);
      const availableHeight=parseFloat(style.maxHeight)-parseFloat(style.paddingTop)-parseFloat(style.paddingBottom);
      fittedWidth=Math.min(availableWidth,availableHeight*Number(display.width)/Number(display.height));
      setZoom(1);
    };
    const toolbar=h('div',{class:'row'},h('button',{'aria-label':'Fit map',title:'Fit the whole map as an overview; zoom for readable cell details',onclick:fit},'Fit'),h('button',{'aria-label':'Zoom map out',onclick:()=>setZoom(zoom-0.5)},'−'),h('button',{'aria-label':'Zoom map in',onclick:()=>setZoom(zoom+0.5)},'+'));
    for(const cell of cells){const kind=terrain(cell.kind),items=list(cell.items).map(obj),agent=items.find(i=>i.namespace==='Agent'),marker=agent?'A':items.some(i=>i.namespace==='Waste outlet')?'O':items.some(i=>i.namespace==='Nest')?'N':items.some(i=>i.namespace==='Spoil')?'S':items.some(i=>i.namespace==='Food')?'F':glyphs[kind]??'?';
      const node=h('button',{class:`spatial-cell ${kind==='Unknown'?'unknown':kind==='KnownSolid'||kind==='Solid'||kind==='#'?'solid':'open'}${items.length?' occupied':''}`,type:'button','data-cell':`${cell.x},${cell.y}`,'aria-label':`Cell ${cell.x}, ${cell.y}: ${valueText(cell.kind)}${items.length?`; ${items.map(i=>`${valueText(i.namespace)} ${i.id===undefined?'':valueText(i.id)}`).join(', ')}`:''}`},marker);
      node.addEventListener('click',()=>{grid.querySelector('[aria-pressed="true"]')?.setAttribute('aria-pressed','false');node.setAttribute('aria-pressed','true');selected.replaceChildren(h('h4',{},`Cell (${cell.x}, ${cell.y})`),fact(display.mapKind==='private remembered topology'?'Remembered classification':'Sampled classification',cell.kind),...items.map(i=>details(`${valueText(i.namespace)}${i.id===undefined?'':` ${valueText(i.id)}`}`,i,true)));});grid.append(node);
    }
    map.append(toolbar,viewport,h('p',{class:'spatial-legend'},'Legend: ? unknown · open # solid · A Agent · N nest · O waste outlet · F Food origin · S Spoil. Food sites are original resource locations; inventory state appears in the inspector. Select cells for exact identities and overlapping items.'));
  }else map.append(h('p',{class:'spatial-unavailable'},'Local map unavailable. This native checkpoint records committed own history without observations, a map, or current holdings.'));
  if(display.overlayCaveat!==null)map.append(h('p',{class:'hint'},'Burrow native ASCII: E exit, o spoil overlay, w/W worker overlay. Glyphs do not identify Agents or precise pile sizes; overlays can hide spoil or multiple workers.'),fact('Native overlay caveat',display.overlayCaveat));
  if(display.mapKind==='private remembered topology')map.append(h('p',{class:'hint'},'Private topology beliefs may be stale. A remembered solid wall can remain solid after the physical terrain opens. Unknown cells carry no inferred terrain, food or occupancy.'));
  map.append(h('p',{class:'hint'},'Only retained sampled checkpoints are shown. No route or action between samples is reconstructed. Fit shows the whole-map overview; zoom or scroll for readable cell details.'));
  for(const a of agents){const local=obj(a.local),state=obj(a.state),pane=h('section',{},h('h4',{},`Agent ${valueText(a.id)}`));
    if(a.state!==null){pane.append(fact('Position',state.pos),fact('Phase',state.phase),...state.mode!==undefined?[fact('Mode',state.mode)]:[],fact('Cargo',state.cargo===null?'No cargo':typeof state.cargo==='string'?{Food:state.cargo}:state.cargo),fact('Own target / site',state.target??state.site),fact('Own captured find',state.find),details('Complete captured own state',a.state));}
    if(local.knowledge){pane.append(fact('Map captured at',local.capture_at),fact('Per-cell observation time',local.cell_observed_at),fact('Knowledge availability',local.knowledge_availability),details('Original private classifications',local.knowledge));}
    if(local.events||local.choices){pane.append(h('p',{class:'hint'},'Historical action origins are committed own traces. They do not establish a current position or current holdings.'),fact('Current holdings',local.holdings),fact('Observations',local.observations),details('Own availability',local.availability),...local.goal?[fact('Supplied known goal',local.goal)]:[],...local.completion_observations?[details('Own completion observations',local.completion_observations)]:[]);
      const history=h('ol',{class:'episode-events'});for(const event of list(local.events))history.append(h('li',{},valueText(event)));pane.append(h('details',{},h('summary',{},`Committed own actions (${history.childElementCount})`),history),details('Own selected choices',local.choices));}
    if(a.local===null&&a.state===null)pane.append(h('p',{},'Local state unavailable at this checkpoint.'));inspector.append(pane);
  }
  if(display.researcher!==null)inspector.append(details('Researcher current native snapshot',obj(display.researcher).snapshot),...obj(display.researcher).access?[details('Researcher access at this checkpoint',obj(display.researcher).access)]:[],details('Researcher sampled resources by namespace',display.resources));
  return h('div',{class:'spatial-view'},h('div',{class:'surface-clock'},fact('Completed clock',display.clock),fact('Native stage',display.stage)),h('div',{class:'spatial-layout'},map,inspector));
}
/** Full native payload is accepted only by the shell's explicit result gate. */
export function renderSpatialResult(payload:Json):HTMLElement {
  const native=obj(obj(payload).native),episode=obj(native.episode),summary=native.summary??native.final_summary??episode.final_summary??native.access??null;
  return h('div',{class:'spatial-result'},h('p',{class:'hint'},'Original native final result. Missing or censored milestones remain unavailable; completed-tick capture clocks differ from processing-tick event clocks.'),details('Native summary / access',summary,true),native.stop_reason??episode.stop_reason?fact('Native stop reason',native.stop_reason??episode.stop_reason):null,details('Complete native result',native),details('Capture metadata',obj(payload).capture));
}
