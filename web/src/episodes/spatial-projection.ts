import type { Checkpoint, Json } from './types';
import { obj, list } from './presentation';
/** Converts only bounded geometry, never identities/counters, to display indexes. */
export function spatialCoordinate(value:Json|undefined):number|null {
  if(typeof value!=='number'&&!(typeof value==='string'&&/^(0|[1-9][0-9]*)$/.test(value)))return null;
  const n=Number(value);return Number.isSafeInteger(n)&&n>=0&&n<=4096?n:null;
}
export function spatialPosition(value:Json|undefined):{x:number;y:number}|null {
  const p=obj(value),x=spatialCoordinate(p.x),y=spatialCoordinate(p.y);return x===null||y===null?null:{x,y};
}
/** Pure formatting of the already permitted checkpoint. No input, future or path source. */
export function spatialDisplay(at:Checkpoint):Json {
  const pub=obj(at.public),researcher=obj(at.researcher),snapshot=obj(researcher.snapshot),grid=obj(researcher.grid);
  const agents=Object.entries(at.local).map(([id,local])=>({id,state:obj(local).agent??null,local}));
  const privateMap=agents.find(a=>obj(a.local).knowledge!==undefined);
  const mapKind=researcher.grid?'native ASCII overlay':at.researcher!==null?'physical state':privateMap?'private remembered topology':agents.some(a=>a.state!==null)?'own state':'unavailable';
  const arena=researcher.grid?grid:obj(pub.arena),width=spatialCoordinate(arena.width),height=spatialCoordinate(arena.height);
  const cells:Json[]=[],resources:Json[]=[];
  if(at.researcher!==null) {
    for(const [field,namespace] of [['resources','Food'],['food','Food'],['spoil','Spoil']] as const) for(const value of list(snapshot[field])) {
      const r=obj(value),original=obj(r.resource??r.food??r.spoil??value);resources.push({namespace,id:original.id??r.id??null,pos:original.pos??r.pos??null,state:r.state??null,native:value});
    }
  }
  const physicalAgents=at.researcher!==null&&!researcher.grid?list(snapshot.agents).map(a=>({id:obj(a).id??null,state:a,local:at.local[String(obj(a).id)]??null})):agents;
  if(width!==null&&height!==null&&width>0&&height>0&&width*height<=4096) {
    const known=new Map<string,Json>();
    if(researcher.grid)for(const c of list(grid.cells).map(obj)){const p=spatialPosition(c);if(p)known.set(`${p.x},${p.y}`,c.glyph??'Unknown');}
    else if(at.researcher!==null&&snapshot.open)for(const p of list(snapshot.open)){const pos=spatialPosition(p);if(pos)known.set(`${pos.x},${pos.y}`,'Open');}
    else if(privateMap)for(const c of list(obj(obj(privateMap.local).knowledge).cells).map(obj)){const p=spatialPosition(c.pos);if(p)known.set(`${p.x},${p.y}`,c.kind??'Unknown');}
    for(let y=0;y<height;y++)for(let x=0;x<width;x++){
      const items:Json[]=[];const same=(p:Json|undefined)=>{const pos=spatialPosition(p);return pos?.x===x&&pos.y===y;};
      for(const nest of Array.isArray(pub.nest)?pub.nest:pub.nest?[pub.nest]:[])if(same(nest))items.push({namespace:'Nest'});
      if(same(pub.waste))items.push({namespace:'Waste outlet'});
      for(const a of physicalAgents)if(same(obj(a.state).pos))items.push({namespace:'Agent',id:a.id});
      for(const r of resources.map(obj))if(same(r.pos))items.push(r);
      const fallback=at.researcher!==null&&!researcher.grid?snapshot.open?'Solid':'Open':'Unknown';
      cells.push({x,y,kind:known.get(`${x},${y}`)??fallback,items});
    }
  }
  return {clock:at.clock,stage:at.kind,mapKind,width,height,cells,agents:physicalAgents,resources,researcher:at.researcher,availability:pub.local_state_availability??null,overlayCaveat:grid.overlay_caveat??null};
}
