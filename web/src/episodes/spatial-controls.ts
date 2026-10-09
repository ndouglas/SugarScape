import { h } from '../ui/dom';
import { canonical } from './comparison';
import { obj, list, label, valueText } from './presentation';
import type { Json, StudyDescriptor } from './types';
/** An opened draft is untouched until a particular edit or named setup selection. */
export class SpatialControls {
  readonly el=h('div',{class:'episode-controls spatial-controls'});
  private draft:Record<string,Json>;
  private jsonEditor:HTMLTextAreaElement|null=null;
  private jsonError:HTMLParagraphElement|null=null;
  private jsonBaseline='';
  private jsonChanged=false;
  constructor(private descriptor:StudyDescriptor,input:Json=descriptor.default_input,private changed:(input:Json)=>void=()=>{}){this.draft=structuredClone(obj(input));this.render();}
  input():Json{this.readCurrentJson();return structuredClone(this.draft);}
  private rootKey():string{return this.descriptor.id.startsWith('burrow')?'config':'setup';}
  /** Action consumers read current text even if no input/change/blur event fired. */
  private readCurrentJson():void {
    const editor=this.jsonEditor;
    if(!editor||(!this.jsonChanged&&editor.value===this.jsonBaseline))return;
    try {
      this.draft[this.rootKey()]=JSON.parse(editor.value) as Json;
      if(this.jsonError)this.jsonError.textContent='';
      editor.removeAttribute('aria-invalid');
    }catch(error){
      this.jsonChanged=true;
      const message=`${this.rootKey()}: ${error instanceof Error?error.message:String(error)}`;
      if(this.jsonError)this.jsonError.textContent=message;
      editor.setAttribute('aria-invalid','true');
      throw new Error(message);
    }
  }
  private edit(change:()=>void):void {
    let valid=true;
    try{this.readCurrentJson();}catch{valid=false;} // The contextual editor error remains visible.
    change();
    if(!valid)return; // Keep unfinished raw text and reject action retrieval until corrected.
    if(this.jsonEditor){
      this.jsonEditor.value=JSON.stringify(this.draft[this.rootKey()],null,2)??'';
      this.jsonBaseline=this.jsonEditor.value;this.jsonChanged=false;
    }
    this.changed(this.input());
  }
  private get(path:string[]):Json|undefined {let value:Json=this.draft;for(const key of path){const row=obj(value);if(!(key in row))return undefined;value=row[key];}return value;}
  private set(path:string[],value:Json):void {this.edit(()=>{let row=this.draft;for(const key of path.slice(0,-1)){if(!row[key]||typeof row[key]!=='object'||Array.isArray(row[key]))row[key]={};row=obj(row[key]);}row[path.at(-1)!]=value;});}
  private scalar(path:string[],title:string,kind:'number'|'text',min?:number,max?:number):HTMLElement {
    const value=this.get(path),numeric=typeof value==='number'&&Number.isFinite(value);
    const input=h('input',{type:kind,value:kind==='number'&&!numeric?'':value===undefined?'':valueText(value),min,max,step:kind==='number'?'any':undefined,'aria-label':title});
    const note=h('span',{class:'hint'},value===undefined?'Missing · validation on run':kind==='number'&&!numeric?`Opened ${valueText(value)} · validation on run`:'');
    if(value===undefined||kind==='number'&&!numeric)input.setAttribute('aria-invalid','true');
    input.addEventListener('change',()=>{this.set(path,kind==='number'&&input.value!==''?Number(input.value):input.value);input.removeAttribute('aria-invalid');note.textContent='';});
    return h('label',{},title,input,note);
  }
  private selector(path:string[],title:string,values:Json[]):HTMLElement {
    const value=this.get(path),picker=h('select',{'aria-label':title},...values.map(v=>h('option',{value:v,selected:v===value},label(valueText(v)))));
    if(!values.includes(value??null)){picker.prepend(h('option',{value:'',selected:true,disabled:true},`Opened ${value===undefined?'missing':valueText(value)} · validation on run`));picker.setAttribute('aria-invalid','true');}
    picker.addEventListener('change',()=>{this.set(path,picker.value);picker.removeAttribute('aria-invalid');});return h('label',{},title,picker);
  }
  private render():void {
    this.el.replaceChildren();
    const scenes=list(obj(this.descriptor.controls).scenes).map(obj),selected=scenes.findIndex(s=>canonical(s.input)===canonical(this.draft));
    const picker=h('select',{'aria-label':'Spatial setup'},h('option',{value:'',selected:selected<0,disabled:true},'Opened/custom input · validation on run'),...scenes.map((s,i)=>h('option',{value:i,selected:i===selected},label(String(s.id)))));
    picker.addEventListener('change',()=>{const scene=scenes[Number(picker.value)];if(scene){this.draft=structuredClone(obj(scene.input));this.render();this.changed(this.input());}});
    this.el.append(h('label',{},'Named engineering setup',picker),h('p',{class:'hint'},'Selecting a named setup explicitly supplies its complete input. Edits affect the next run; native validation checks the opened values.'));
    const common=h('div',{class:'row'},this.scalar(['seed'],'Seed (decimal u64)','text'),this.scalar(['ticks'],'Requested ticks','number',this.descriptor.id.startsWith('burrow')?0:1),this.scalar(['sample_every'],'Sample every','number',1));this.el.append(common);
    const controls=obj(this.descriptor.controls),parameters=h('div',{class:'spatial-parameters'});
    for(const [key,value] of Object.entries(controls)){const spec=obj(value),path=list(spec.path);if(!path.length)continue;const strings=path.map(String),cpfa=this.descriptor.id.startsWith('foraging_');parameters.append(cpfa?this.scalar(strings,label(key),'number',0,key.startsWith('p_')?1:key==='omega'?4*Math.PI:['lambda_fidelity','lambda_publish'].includes(key)?256:undefined):this.selector(strings,label(key),list(spec.values)));}
    this.el.append(parameters);
    if(this.descriptor.id.startsWith('burrow')){
      const base=this.descriptor.id==='burrow_access'?['config','lab']:['config'];
      const tuning=h('div',{class:'spatial-parameters'},this.scalar([...base,'freshness_window'],'Freshness window (decimal u64)','text'));
      for(const key of ['relay_distance','response_weight','minimum_recent_units'])tuning.append(this.scalar([...base,key],label(key),'number',0));
      if(this.descriptor.id==='burrow_access')for(const key of ['x','y'])tuning.append(this.scalar(['config','task','goal',key],`Goal ${key}`,'number',0));
      if(this.descriptor.id==='burrow_access')tuning.append(this.scalar(['config','task','goal_weight'],'Goal weight','number',0));
      this.el.append(h('details',{},h('summary',{},'Original Burrow settings'),tuning));
    }
    const root=this.rootKey();
    const json=h('textarea',{'aria-label':`Complete ${root} JSON`,rows:8,value:this.draft[root]===undefined?'':JSON.stringify(this.draft[root],null,2),spellcheck:false});
    this.jsonEditor=json;this.jsonBaseline=json.value;this.jsonChanged=false;
    const error=h('p',{role:'status',class:'hint'});this.jsonError=error;
    json.addEventListener('input',()=>{this.jsonChanged=true;});
    json.addEventListener('change',()=>{
      this.jsonChanged=true;
      let input:Json;
      try{input=this.input();}catch{return;} // input() has shown the contextual parse error.
      this.render();this.changed(input);
    });
    const geometry=h('details',{},h('summary',{},'Geometry, resources and complete native settings'),h('p',{class:'hint'},'JSON retains spawn order, geometry, resource identities and all original fields. Resource IDs must be quoted decimal strings.'),json,error);
    const setup=obj(this.draft.setup),field=this.descriptor.id==='foraging_construction'?'food':'resources';
    for(const [index,resource] of list(setup[field]).entries()){
      const id=h('input',{type:'text',value:valueText(obj(resource).id),'aria-label':`${field} ${index} ID (decimal u64)`});
      id.addEventListener('change',()=>this.edit(()=>{const resources=list(obj(this.draft.setup)[field]);const row=obj(resources[index]);row.id=id.value;}));geometry.append(h('label',{},`${label(field)} ${index} ID (decimal u64)`,id));
    }
    this.el.append(geometry);
  }
}
