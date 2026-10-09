import { SpatialControls } from './spatial-controls';
import { spatialCatalog,renderDescriptor } from './catalog';
import { renderSpatial,renderSpatialResult } from './spatial-view';
import { obj,list } from './presentation';
import type { EpisodeView } from './view';
import type { StudyDescriptor } from './types';
export interface BrowserCheck {name:string;passed:boolean}
export function spatialDomChecks(catalog:StudyDescriptor[]):BrowserCheck[] {
 const checks:BrowserCheck[]=[];const check=(name:string,passed:boolean)=>checks.push({name,passed});
 for(const native of catalog.filter(d=>d.family==='spatial')){
  const d=spatialCatalog(native),input=structuredClone(obj(d.default_input));
  if(d.id.startsWith('foraging_'))check(`${d.id}: loaded UI axes use zero/one with native field definitions preserved`,list(obj(d.controls).comparisons).every(a=>JSON.stringify(obj(a).values)==='[0,1]')&&Object.entries(obj(native.controls)).every(([key,value])=>JSON.stringify(obj(d.controls)[key])===JSON.stringify(value)));
  input.seed='bad';delete input.sample_every;
  const controls=new SpatialControls(d,input);document.body.append(controls.el);
  check(`${d.id}: missing/malformed draft remains exact`,JSON.stringify(controls.input())===JSON.stringify(input));
  check(`${d.id}: seed remains text`,controls.el.querySelector<HTMLInputElement>('[aria-label="Seed (decimal u64)"]')?.type==='text');
  const picker=controls.el.querySelector<HTMLSelectElement>('[aria-label="Spatial setup"]');
  check(`${d.id}: opened custom setup selected`,picker?.value==='');
  if(picker){picker.value='0';picker.dispatchEvent(new Event('change'));check(`${d.id}: only explicit scene selection supplies input`,JSON.stringify(controls.input())===JSON.stringify(obj(list(obj(d.controls).scenes)[0]).input));}
  controls.el.remove();
 }
 const d=spatialCatalog(catalog.find(d=>d.id==='foraging_construction')!);
 const at={index:0,kind:'spatial_sample',clock:{completed_ticks:'8'},public:{arena:{width:'3',height:'2'},nest:[{x:'0',y:'0'}],local_state_availability:'captured own state'},local:{'1':{agent:{id:'1',pos:{x:'0',y:'0'},cargo:{Spoil:'0'}},knowledge:{cells:[{pos:{x:'1',y:'0'},kind:{KnownSolid:{diggable:true}}}]},capture_at:{completed_ticks:'8'},cell_observed_at:null}},researcher:null};
 const map=renderSpatial(at,renderDescriptor(d));document.body.append(map);
 check('private map says memories may be stale',map.textContent!.includes('stale'));
 check('capture time is not a cell learning age',map.textContent!.includes('Per-cell observation time: Unavailable'));
 check('no global food or peer position appears',!map.textContent!.includes('Global food'));
 const cell=map.querySelector<HTMLButtonElement>('[data-cell="1,0"]');cell?.click();
 check('selection exposes exact tagged remembered wall',map.textContent!.includes('diggable: true'));
 const zoom=map.querySelector<HTMLButtonElement>('[aria-label="Zoom map in"]');zoom?.click();
 check('zoom responds in actual DOM',Number(map.querySelector<HTMLElement>('.spatial-grid')?.dataset.zoom)>1);
 map.querySelector<HTMLButtonElement>('[aria-label="Fit map"]')?.click();check('fit restores map zoom',map.querySelector<HTMLElement>('.spatial-grid')?.dataset.zoom==='1');const viewport=map.querySelector<HTMLElement>('.spatial-viewport'),grid=map.querySelector<HTMLElement>('.spatial-grid');const vr=viewport?.getBoundingClientRect(),gr=grid?.getBoundingClientRect();check('fit places whole grid inside visible viewport',!!vr&&!!gr&&gr.right<=vr.right&&gr.bottom<=vr.bottom);map.remove();
 const result=renderSpatialResult({native:{summary:{first_delivery_tick:null}}});check('censored outcome displays unavailable',result.textContent!.includes('Unavailable'));
 return checks;
}
export async function spatialViewChecks(view:EpisodeView,catalog:StudyDescriptor[]):Promise<BrowserCheck[]> {
 const checks:BrowserCheck[]=[];const check=(name:string,passed:boolean)=>checks.push({name,passed});
 const button=(text:string)=>[...view.el.querySelectorAll('button')].find(b=>b.textContent===text)!;
 const idle=async()=>{const deadline=performance.now()+60000;while(view.el.querySelector<HTMLFieldSetElement>('.episode-setup')!.disabled){if(performance.now()>deadline)throw new Error('Run did not settle');await new Promise(r=>setTimeout(r,20));}};
 await view.ready;
 for(const d of catalog){view.openInput(d.default_input);button('Run episode').click();await idle();check(`${d.id}: default renders native successful record`,view.session.shown?.study===d.id&&view.el.querySelector('.episode-rendered')!.textContent!.length>0);if(d.family!=='spatial')continue;
 const original=view.session.shown!;check(`${d.id}: real map or explicit unavailable map`,!!view.el.querySelector('.spatial-view'));
 const perspective=view.el.querySelector<HTMLSelectElement>('[aria-label="Perspective"]')!;perspective.focus();view.replay.step();check(`${d.id}: perspective focus survives draw`,document.activeElement===perspective);perspective.value='researcher';perspective.dispatchEvent(new Event('change'));check(`${d.id}: Researcher map visible`,!!view.el.querySelector('.spatial-grid'));perspective.value=`agent:${Object.keys(original.checkpoints[0].local)[0]}`;perspective.dispatchEvent(new Event('change'));
 const axis=view.el.querySelector<HTMLSelectElement>('[aria-label="Spatial comparison axis"]')!;axis.focus();const axisValue=axis.value;view.replay.step();check(`${d.id}: axis focus and selection survive draw`,document.activeElement===axis&&axis.value===axisValue);
 button('Run matched comparison').click();await idle();check(`${d.id}: two successful matched records retained`,view.session.records.length===2&&view.session.shown===original);
 const seek=view.el.querySelector<HTMLInputElement>('[aria-label="Comparison checkpoint"]')!;seek.focus();seek.value='1';seek.dispatchEvent(new Event('input'));check(`${d.id}: comparison slider focus stable`,document.activeElement===seek&&seek.value==='1');
 const successful=view.session.shown,records=[...view.session.records];view.openInput({...obj(d.default_input),seed:'bad'});button('Run episode').click();await idle();check(`${d.id}: failed run retains original comparison`,view.session.shown===successful&&records.every((r,i)=>view.session.records[i]===r));
 }
 return checks;
}

/** Actual mounted UI lifecycle with real workers and one intentionally nonreply worker. */
export async function spatialLifecycleChecks(catalog:StudyDescriptor[]):Promise<BrowserCheck[]> {
 const {EpisodeView}=await import('./view');const {createEpisodeClient}=await import('./client');
 const checks:BrowserCheck[]=[];const check=(name:string,passed:boolean)=>checks.push({name,passed});
 let mode='normal',live=0,peak=0,created=0,terminated=0;
 const blob=URL.createObjectURL(new Blob(['self.onmessage=()=>{};'],{type:'text/javascript'}));
 const client=createEpisodeClient(()=>{const worker=mode==='timeout'?new Worker(blob):new Worker(new URL('./worker.ts',import.meta.url),{type:'module'});live++;created++;peak=Math.max(peak,live);const terminate=worker.terminate.bind(worker);worker.terminate=()=>{live--;terminated++;terminate();};return worker;});
 const view=new EpisodeView(client);document.body.append(view.el);await view.ready;
 const button=(text:string)=>[...view.el.querySelectorAll('button')].find(b=>b.textContent===text)!;
 const idle=async()=>{const deadline=performance.now()+65000;while(view.el.querySelector<HTMLFieldSetElement>('.episode-setup')!.disabled){if(performance.now()>deadline)throw new Error('Lifecycle UI did not settle');await new Promise(r=>setTimeout(r,20));}};
 const d=catalog.find(d=>d.id==='foraging_fixed')!;view.openInput(d.default_input);button('Run episode').click();await idle();const first=view.session.shown!;
 view.openInput({...obj(d.default_input),seed:'18446744073709551615'});button('Run episode').click();await idle();const original=view.session.shown!;
 const exact=await view.session.export();check('real export validates complete same-target record',JSON.stringify(JSON.parse(exact))===JSON.stringify(original));
 const file=view.el.querySelector<HTMLInputElement>('input[type="file"]')!,transfer=new DataTransfer();transfer.items.add(new File([exact],'task6-episode.json',{type:'application/json'}));file.files=transfer.files;file.dispatchEvent(new Event('change'));await idle();check('real file open imports validated native result',JSON.stringify(view.session.shown)===exact);
 const successful=view.session.shown!,retained=[...view.session.records];const bad=new DataTransfer();bad.items.add(new File([exact.replace('spatial-adapter-1','spatial-adapter-bad')],'task6-forged.json',{type:'application/json'}));file.files=bad.files;file.dispatchEvent(new Event('change'));await idle();check('failed import retains prior successful records',view.session.shown===successful&&retained.every((r,i)=>view.session.records[i]===r));
 // Cancellation during actual worker initialization; the successful record stays shown.
 button('Run episode').click();button('Cancel').click();await idle();check('UI cancellation retains successful record',view.session.shown===successful&&view.replay.record===successful);
 // A retained selector changes the operation generation during a real comparison.
 button('Run matched comparison').click();const picker=view.el.querySelector<HTMLSelectElement>('[aria-label="Retained episode"]')!;picker.value='0';picker.dispatchEvent(new Event('change'));const switched=view.replay.record;await idle();check('stale comparison cannot replace switched original',view.replay.record===switched&&view.session.records.every((r,i)=>retained[i]===r));
 const perspective=view.el.querySelector<HTMLSelectElement>('[aria-label="Perspective"]')!;
 const axis=view.el.querySelector<HTMLSelectElement>('[aria-label="Spatial comparison axis"]')!;axis.focus();view.replay.reset();const initial=view.replay.index;view.replay.play();const deadline=performance.now()+2000;while(view.replay.index===initial&&performance.now()<deadline)await new Promise(r=>setTimeout(r,20));view.pause();check('axis focus and node survive autoplay redraw',document.activeElement===axis&&view.el.querySelector('[aria-label="Spatial comparison axis"]')===axis);check('autoplay advances recorded checkpoints',view.replay.index>initial||matchMedia('(prefers-reduced-motion: reduce)').matches);
 perspective.focus();const before=view.replay.index;view.replay.step();check('perspective control remains stable through seek',document.activeElement===perspective&&view.replay.index>=before);
 view.openInput({...obj(d.default_input),setup:{...obj(obj(d.default_input).setup),parameters:{...obj(obj(obj(d.default_input).setup).parameters),lambda_publish:-1}}});button('Run episode').click();await idle();check('unsupported parameter rejected without replacing shown record',view.replay.record===switched&&view.el.querySelector(':scope > .row > [role="status"]')!.textContent!.includes('lambda_publish'));
 mode='timeout';view.openInput(d.default_input);button('Run episode').click();await idle();check('real 60-second worker timeout retains prior record',view.replay.record===switched&&view.el.querySelector(':scope > .row > [role="status"]')!.textContent!.includes('60 seconds'));check('only one worker outstanding and all released',peak===1&&live===0&&created===terminated);
 view.dispose();check('dispose clears editing, record and retained references',view.session.editing===null&&view.session.shown===null&&view.session.records.length===0&&view.replay.record===null&&view.el.querySelector('.episode-controls')===null);view.el.remove();URL.revokeObjectURL(blob);
 // The native descriptor stays valid while a test catalog has no compatible surface choice.
 const emptyClient=createEpisodeClient();const surface=catalog.find(d=>d.id==='active_surface')!;
 const emptyView=new EpisodeView({...emptyClient,catalog:async()=>[{...surface,controls:{...obj(surface.controls),settings:[]}}]});document.body.append(emptyView.el);await emptyView.ready;
 [...emptyView.el.querySelectorAll('button')].find(b=>b.textContent==='Run episode')!.click();const limit=performance.now()+60000;while(emptyView.el.querySelector<HTMLFieldSetElement>('.episode-setup')!.disabled){if(performance.now()>limit)throw new Error('Empty-choice fixture did not settle');await new Promise(r=>setTimeout(r,20));}
 const compare=[...emptyView.el.querySelectorAll('button')].find(b=>b.textContent==='Run matched comparison')!;check('empty candidate comparison stays disabled after busy release',compare.disabled);emptyView.dispose();emptyView.el.remove();
 check('source default record remains unchanged',JSON.stringify(first.input)===JSON.stringify(d.default_input));return checks;
}
