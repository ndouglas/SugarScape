import scenes from '../../../crates/sugarscape-core/src/browser_experiments/fixtures/caching-scenes.json';
import spatialScenes from '../../../crates/sugarscape-core/src/browser_experiments/fixtures/spatial-scenes.json';
import { CachingControls } from './caching-controls';
import { SpatialControls } from './spatial-controls';
import { spatialCatalog,renderDescriptor } from './catalog';
import { renderCaching } from './caching-view';
import { EpisodeView } from './view';
import { projectCheckpoint } from './projection';
import { decodeEpisode } from './share';
import { obj,list } from './presentation';
import type { Json,StudyDescriptor,EpisodeClient,Checkpoint } from './types';
export interface CachingBrowserCheck {name:string;passed:boolean}
/** Explicit UI fixtures: no native/WASM parity claim. Runnable in the inherited QA harness. */
export async function cachingDomChecks():Promise<CachingBrowserCheck[]> {
 const checks:CachingBrowserCheck[]=[],check=(name:string,passed:boolean)=>checks.push({name,passed});
 const catalog=list(obj(scenes as unknown as Json).scenes).filter((scene,index,rows)=>rows.findIndex(s=>obj(obj(s).input).study===obj(obj(scene).input).study)===index).map(scene=>{const input=obj(scene).input,id=String(obj(input).study);return spatialCatalog({id,family:'spatial',title:id,supplied:'Test fixture',question:'Test fixture',default_input:input,controls:obj(obj(obj(scenes as unknown as Json).studies)[id]).controls} as StudyDescriptor);});
 const checkpoint=(lab:string):Checkpoint=>({index:0,kind:'caching_initial',clock:{native_tick:'0',action_interval:null},public:{lab_kind:lab,arena:{width:9,height:9},layout_kind:'reference_layout',walls:[{x:0,y:0}]},local:{'1':{agent:{id:'1',pos:{x:2,y:3},holdings:0,caches:[{site:29,pos:{x:2,y:3},amount:0}]},seen:[{site:30,pos:{x:3,y:3},owner:'2',amount:8,tick:'0',age:'0'}],own_actions:[],...(lab==='protection'?{protection:{exposure_span:'0',sources:[],exposure:{entries:[{site:30,pos:{x:3,y:3},age:'1',expired:true}]}}}:{sender:null,received_observations:[{actor:'2',site:'30',tick:'0',signal:{visible_transfer:{amount:0}}}]}),availability:'captured'},'2':{agent:{id:'2',pos:{x:7,y:7},holdings:'private-peer-sentinel'}}},researcher:{frame:{actual_transfer:'hidden-transfer-sentinel',actual_stock:'hidden-stock-sentinel'},display:{stocks:[{site:30,pos:{x:3,y:3},amount:1}]}}});
 let requests=0,copies:string[]=[],cancels=0,disposals=0,holdNextRequest=false;
 let rejectPending:((error:Error)=>void)|null=null;
 const client:EpisodeClient={catalog:async()=>catalog,recorded:async()=>null,cancel:()=>{cancels++;const reject=rejectPending;rejectPending=null;reject?.(new Error('fixture canceled'));},dispose:()=>{disposals++;rejectPending?.(new Error('fixture disposed'));rejectPending=null;},request:async(_op,text)=>{requests++;if(holdNextRequest){holdNextRequest=false;await new Promise<never>((_resolve,reject)=>{rejectPending=reject;});}const input=JSON.parse(text) as Json,d=catalog.find(d=>d.id===obj(input).study)!;if(obj(input).seed==='bad')throw new Error('seed: invalid');const at=checkpoint(String(obj(d.controls).lab_kind));return {kind:'experiment_episode',version:1,study:d.id,rules_identity:'ui-fixture',input,semantics:'trajectory',checkpoints:[at,{...at,index:1,kind:'caching_terminal',clock:{native_tick:'1'}}],payload:{terminal:'terminal-sentinel'}};}};
 const originalWrite=navigator.clipboard.writeText;navigator.clipboard.writeText=async text=>{copies.push(text);};
 const view=new EpisodeView(client);document.body.append(view.el);await view.ready;
 const button=(name:string)=>[...view.el.querySelectorAll('button')].find(b=>b.textContent===name)!;
 const settle=async()=>{for(let i=0;i<100&&view.el.querySelector<HTMLFieldSetElement>('.episode-setup')!.disabled;i++)await new Promise(r=>setTimeout(r,10));};
 try{
 for(const d of catalog){
  const controls=new CachingControls(d);check(`${d.id}: no horizon/sampling controls`,!controls.el.querySelector('[aria-label="Requested ticks"]')&&!controls.el.querySelector('[aria-label="Sample every"]')&&controls.el.textContent!.includes('64 requested ticks'));
  if(d.id==='protection_recaching')check('P3 observer span native domain begins at one',controls.el.querySelector<HTMLInputElement>('[aria-label="Observer span"]')?.min==='1');
  const opened={...obj(d.default_input),seed:'18446744073709551615',lab:{...obj(obj(d.default_input).lab),unexpected:'retain'}};check(`${d.id}: malformed/missing custom fields retained`,JSON.stringify(new CachingControls(d,opened).input())===JSON.stringify(opened));
  view.openInput(d.default_input);button('Run episode').click();await settle();const shown=view.session.shown!,retained=[...view.session.records];
  const text=view.el.querySelector<HTMLTextAreaElement>('[aria-label="Complete lab JSON"]')!;text.value='{x';const before=requests,beforeCopies=copies.length;
  button('Run episode').click();await settle();button('Share input link').click();await new Promise(r=>setTimeout(r,10));
  check(`${d.id}: malformed before blur blocks Run/Share`,requests===before&&copies.length===beforeCopies&&text.getAttribute('aria-invalid')==='true');
  check(`${d.id}: malformed actions retain successful shown records`,view.session.shown===shown&&view.replay.record===shown&&retained.every((r,i)=>view.session.records[i]===r));
  const seed=view.el.querySelector<HTMLInputElement>('[aria-label="Seed (decimal u64)"]')!;seed.value='18446744073709551615';seed.dispatchEvent(new Event('change'));check(`${d.id}: unrelated edit preserves malformed text`,text.value==='{x');
  text.value=JSON.stringify(obj(d.default_input).lab);button('Run episode').click();await settle();check(`${d.id}: current valid text recovers without blur`,view.session.shown!==shown&&obj(view.session.shown!.input).seed==='18446744073709551615');
  button('Share input link').click();for(let i=0;i<100&&copies.length===beforeCopies;i++)await new Promise(r=>setTimeout(r,10));const token=copies.at(-1)?.split('#e=')[1];check(`${d.id}: Share uses current seed and lab`,!!token&&obj(await decodeEpisode(token!)).seed==='18446744073709551615');
  const map=renderCaching(projectCheckpoint(shown,0,{kind:'agent',agent:'1'}),renderDescriptor(d));document.body.append(map);
  check(`${d.id}: reference map hides peer/research/future/terminal`,!map.textContent!.match(/private-peer-sentinel|hidden-transfer-sentinel|hidden-stock-sentinel|terminal-sentinel/)&&map.textContent!.includes('setup scaffolding'));
  map.querySelector<HTMLButtonElement>('[data-cell="3,3"]')!.click();check(`${d.id}: selected evidence tick/age and clear zero`,map.textContent!.includes('amount: 8')&&map.textContent!.includes('age: 0')&&(d.id==='protection_recaching'||map.textContent!.includes('visible transfer: amount: 0')));
  map.querySelector<HTMLButtonElement>('[aria-label="Zoom map in"]')!.click();check(`${d.id}: inherited zoom works`,map.querySelector<HTMLElement>('.spatial-grid')!.dataset.zoom==='1.5');map.querySelector<HTMLButtonElement>('[aria-label="Fit map"]')!.click();check(`${d.id}: inherited fit restores full grid`,map.querySelector<HTMLElement>('.spatial-grid')!.dataset.zoom==='1');map.remove();
  const perspective=view.el.querySelector<HTMLSelectElement>('[aria-label="Perspective"]')!;perspective.focus();view.replay.step();check(`${d.id}: focus survives seek`,document.activeElement===perspective);view.replay.reset();view.replay.step();check(`${d.id}: repeated seek renders same boundary`,view.replay.index===1);
  view.openInput({...obj(d.default_input),seed:'bad'});const success=view.session.shown;button('Run episode').click();await settle();check(`${d.id}: failed replacement retains successful record`,view.session.shown===success);
  const original=view.session.shown;button('Run matched comparison').click();await settle();check(`${d.id}: matched comparison retains two successful records`,view.session.records.length===2&&view.session.shown===original);
  view.pause();check(`${d.id}: navigation pause retains record`,view.replay.record===success&&!view.replay.playing);
 }
 const beforeCancel=view.session.shown,retainedBeforeCancel=[...view.session.records],requestCount=requests,cancelCount=cancels;
 view.openInput(catalog[0].default_input);holdNextRequest=true;button('Run episode').click();
 check('cancel fixture has a genuinely pending UI request',requests===requestCount+1&&rejectPending!==null&&view.el.querySelector<HTMLFieldSetElement>('.episode-setup')!.disabled&&!button('Cancel').disabled);
 button('Cancel').click();await settle();
 check('UI cancellation invokes client and settles pending request',cancels===cancelCount+1&&rejectPending===null&&!view.el.querySelector<HTMLFieldSetElement>('.episode-setup')!.disabled&&button('Cancel').disabled);
 check('UI cancellation retains successful record and retained comparison',view.session.shown===beforeCancel&&view.replay.record===beforeCancel&&retainedBeforeCancel.length===view.session.records.length&&retainedBeforeCancel.every((record,i)=>record===view.session.records[i]));
 // Both affected legacy editor roots use the same current-text parser.
 for(const id of ['foraging_fixed','burrow_excavation']){const scene=list(obj(spatialScenes as unknown as Json).scenes).find(s=>obj(obj(s).input).study===id)!,input=obj(scene).input,root=id.startsWith('burrow')?'config':'setup';const d=spatialCatalog({id,family:'spatial',title:id,supplied:'',question:'',default_input:input,controls:{}} as StudyDescriptor),control=new SpatialControls(d,input);const text=control.el.querySelector<HTMLTextAreaElement>(`[aria-label="Complete ${root} JSON"]`)!;text.value='{x';let threw=false;try{control.input();}catch{threw=true;}const seed=control.el.querySelector<HTMLInputElement>('[aria-label="Seed (decimal u64)"]')!;seed.value='23';seed.dispatchEvent(new Event('change'));check(`${root}: legacy invalid-before-blur/unrelated edit`,threw&&text.value==='{x');text.value=JSON.stringify(obj(input)[root]);check(`${root}: legacy recovery retains seed`,obj(control.input()).seed==='23');}
 }finally{navigator.clipboard.writeText=originalWrite;view.dispose();check('disposal clears retained/editing/replay references',view.session.records.length===0&&view.session.shown===null&&view.session.editing===null&&view.replay.record===null&&disposals===1);view.el.remove();}
 return checks;
}
