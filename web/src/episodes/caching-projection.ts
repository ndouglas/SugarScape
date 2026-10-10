import type { Checkpoint,Json } from './types';
import { obj,list } from './presentation';
import { spatialPosition } from './spatial-projection';
/** Only a validated bounded site is converted; identities and clocks remain text. */
export function cachingSitePosition(value:Json|undefined):{x:number;y:number}|null {
  if(typeof value!=='number'&&!(typeof value==='string'&&/^(0|[1-9][0-9]*)$/.test(value)))return null;
  const site=Number(value);return Number.isInteger(site)&&site>=0&&site<81?{x:site%9,y:Math.floor(site/9)}:null;
}
/** Pure projection of an already perspective-selected checkpoint. */
export function cachingDisplay(at:Checkpoint):Json {
  const pub=obj(at.public),researcher=obj(at.researcher);
  const agents=Object.entries(at.local).filter(([,local])=>obj(local).agent!==undefined).map(([id,local])=>({id,state:obj(local).agent!,local}));
  const layers:Json[]=[];
  for(const a of agents){const local=obj(a.local),state=obj(a.state);
    for(const cache of list(state.caches))layers.push({namespace:'Own buried cache',owner:a.id,...obj(cache)});
    for(const evidence of list(local.seen))layers.push({namespace:'Remembered cache evidence',...obj(evidence)});
    for(const observation of list(local.received_observations)){const row=obj(observation),pos=cachingSitePosition(row.site);if(pos)layers.push({namespace:'Received public observation',...row,pos});}
  }
  if(at.researcher!==null)for(const stock of list(obj(researcher.display).stocks))layers.push({namespace:'Surface resource',...obj(stock)});
  const cells:Json[]=[];
  for(let y=0;y<9;y++)for(let x=0;x<9;x++){
    const same=(p:Json|undefined)=>{const pos=spatialPosition(p);return pos?.x===x&&pos.y===y;};
    const items:Json[]=agents.filter(a=>same(obj(a.state).pos)).map(a=>({namespace:'Agent',id:a.id}));
    items.push(...layers.filter(layer=>same(obj(layer).pos)));
    cells.push({x,y,kind:list(pub.walls).some(same)?'Solid':'Open',items});
  }
  return {clock:at.clock,stage:at.kind,layout_kind:'reference_layout',mapKind:'Reference layout',width:9,height:9,cells,agents,researcher:at.researcher,availability:agents.length?'Captured surviving role state':'Selected Agent absent at this boundary; no local state captured',overlayCaveat:null};
}
