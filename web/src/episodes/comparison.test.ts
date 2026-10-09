import { expect, it } from 'vitest';
import { comparisonKey, withActivePolicy, matchedInputs, matchingCheckpoint } from './comparison';
import type { StudyDescriptor, Checkpoint } from './types';
const input={study:'active_surface',environment:'DataFlip',sequence:0,protocol:{experimenter:'A',policy:'Adaptive',ids:{agents:['one','two'],surface:'field'}}};
it('clones nested policy only without modifying source',()=>{
 const next=withActivePolicy(input,'NoProbe');
 expect(next).toEqual({...input,protocol:{...input.protocol,policy:'NoProbe'}});
 expect(input.protocol.policy).toBe('Adaptive');
 expect(next.protocol).not.toBe(input.protocol);
 expect(()=>withActivePolicy(input,'no_probe')).toThrow();
});
it('canonical comparison excludes only nested policy and retains all science and IDs',()=>{
 expect(comparisonKey(withActivePolicy(input,'NoProbe'))).toBe(comparisonKey(input));
 for(const changed of [{...input,sequence:255},{...input,environment:'Inert'},{...input,protocol:{...input.protocol,experimenter:'B'}},{...input,protocol:{...input.protocol,ids:{agents:['x','y'],surface:'field'}}}])expect(comparisonKey(changed)).not.toBe(comparisonKey(input));
 expect(comparisonKey({...input,protocol:{...input.protocol,policy:'Adaptive',extra:1}})).not.toBe(comparisonKey(input));
 expect(comparisonKey({protocol:input.protocol,sequence:0,environment:'DataFlip',study:'active_surface'})).toBe(comparisonKey(input));
});
it('catalog matching refuses incompatible setup and preserves custom IDs',()=>{
 const d={id:'active_surface',controls:{settings:[{environment:'DataFlip',protocol:{...input.protocol,policy:'NoProbe',ids:{agents:['A','B'],surface:'default'}}},{environment:'DataFlip',protocol:{...input.protocol,policy:'Known',experimenter:'B'}}]}} as unknown as StudyDescriptor;
 expect(matchedInputs(input,d)).toEqual([withActivePolicy(input,'NoProbe')]);
});
it('shared matches preserve calibration prior and world while changing only pair',()=>{
 const shared={study:'shared_surface',sequence:255,environment:'DataFlip',protocol:{calibration_rounds:3,prior_mode:'Treatment',pair:{roles:['Unknown','Unknown']},ids:{agents:['one','two'],surface:'field'}}};
 expect(comparisonKey({...shared,protocol:{...shared.protocol,pair:{roles:['Known','Known']}}})).toBe(comparisonKey(shared));
 expect(comparisonKey({...shared,protocol:{...shared.protocol,calibration_rounds:0}})).not.toBe(comparisonKey(shared));
 expect(comparisonKey({...shared,protocol:{...shared.protocol,prior_mode:'RestartUniform'}})).not.toBe(comparisonKey(shared));
});
it('joins native public clocks and kind rather than array positions',()=>{
 const at:Checkpoint={index:0,public:{},local:{},researcher:null,clock:{AfterSlot:{round:'3',slot:'1'}},kind:'after_slot'};
 const points=[{...at,clock:'Finished'},at];
 expect(matchingCheckpoint(points,at)).toBe(1);
 expect(matchingCheckpoint([points[0]],at)).toBe(-1);
 expect(matchingCheckpoint([{...at,kind:'before_slot'}],at)).toBe(-1);
});
