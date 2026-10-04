"""Partial configurations and fixed jobs; Rust alone resolves engine defaults."""
from copy import deepcopy
import hashlib
import json
import numpy as np
from .methods import canonical_bytes, contract_sha256, method_contract

CONTROLS = (
 ('printed_increasing','distance_formula','printed_increasing'),
 ('manhattan','distance_metric','manhattan'),
 ('attacker_first','path_sampling','attacker_first'),
 ('global_no_action','initiation_guard','global_no_action'),
 ('all_fronts','enemy_total','all_fronts'),
 ('own_commitment','damage_basis','own_commitment'),
 ('exclusive','victory_draws','exclusive'),
 ('collapse_only','capital_capture','collapse_only'),
 ('affected_states','locking','affected_states'),
 ('adjacent_active_states','cluster_linkage','adjacent_active_states'),
 ('add_losses','damage_feedback','add_losses'),
 ('initiator_curve','attack_projection','initiator_curve'),
 ('mutual_only','severity_damage','mutual_only'),
 ('java_int100','severity_export','java_int100'),
)

TARGETS=('slope_min','slope_median','slope_max','r2_min','r2_median','r2_max','log_range_median','war_count_median')


def build_manifest(source,source_bytes=None):
    rows=source.get('rows',[])
    ids=[r.get('id') for r in rows]
    if len(rows)!=11 or len(set(ids))!=11 or [r.get('row') for r in rows]!=list(range(1,12)):
        raise ValueError('source table must have eleven unique ordered rows')
    if any(r.get('source_histories')!=15 or set(r.get('targets',{}))!=set(TARGETS) for r in rows):
        raise ValueError('source table must have fifteen histories and eight targets per row')
    arms=[]
    def add(family,name,partial,n,preset='paper'):
        index=len(arms)
        arms.append({'id':f'{family}.{name}','family':family,'index':index,'preset':preset,
                     'config_overrides':{'periods_per_tick':100,**deepcopy(partial)},
                     'sessions':n,'first_seed':370000001+index*10000})
    for family,n in [('original',15),('precision',100)]:
        for row in rows:add(family,row['id'],row['config_overrides'],n)
    for name,field,value in CONTROLS:add('reading',name,{field:value},15)
    add('artifact','reference_2017',{},15,'artifact_2017')
    if source_bytes is not None and json.loads(source_bytes)!=source:
        raise ValueError('source table bytes do not match parsed source')
    manifest={'schema_version':1,'model':'geosim','execution_mode':'registered',
              'provenance_status':'provisional_unfrozen',
              'spec':'docs/superpowers/specs/2026-10-03-geosim-design.md',
              'source_draws':100000,'ks_draws':1000,'analysis_seed':2026100301,
              'source_table_sha256':None if source_bytes is None else hashlib.sha256(source_bytes).hexdigest(),
              'method_contract':method_contract(),'method_contract_json':canonical_bytes(method_contract()).decode('utf-8'),
              'method_contract_sha256':contract_sha256(),
              'source_inventory':None,'source_inventory_sha256':None,'arms':arms}
    manifest['analysis_jobs']=analysis_jobs(manifest)
    return manifest


def expected_keys(manifest):
    result=[]
    for index,a in enumerate(manifest['arms']):
        if a['index']!=index or a['sessions'] not in (15,100) or a['first_seed']!=370000001+index*10000:
            raise ValueError('registered arm key contract changed')
        result.extend((a['id'],a['first_seed']+r) for r in range(a['sessions']))
    if len(manifest['arms'])!=37 or len(result)!=1490 or len(set(result))!=1490:
        raise ValueError('registered workload must contain37 arms/1490 unique keys')
    return result


def analysis_jobs(manifest):
    arms=manifest['arms']
    ids=[f"predictive.source.{a['id'].split('.',1)[1]}" for a in arms if a['family']=='original']
    ids+=['contrast.base_minus_shock0','contrast.base_minus_context_off']
    ids+=[f"ks.{a['id']}" for a in arms if a['family'] in ('original','precision')]
    ids+=[f"parameters.{a['id']}" for a in arms]
    if len(ids)!=72 or len(set(ids))!=72:raise ValueError('fixed analysis job family changed')
    return [{'index':i,'id':job,'root_entropy':2026100301,'spawn_key':[i],
             'state_u32':np.random.SeedSequence(2026100301,spawn_key=(i,)).generate_state(4).tolist()}
            for i,job in enumerate(ids)]


def job_rng(job):
    if job['root_entropy']!=2026100301 or job['spawn_key']!=[job['index']]:
        raise ValueError('analysis seed derivation mismatch')
    seed=np.random.SeedSequence(2026100301,spawn_key=(job['index'],))
    if job.get('state_u32')!=seed.generate_state(4).tolist():raise ValueError('child seed state mismatch')
    return np.random.Generator(np.random.PCG64(seed))


def main():
    import argparse
    from pathlib import Path
    from .records import strict_json
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source',required=True,type=Path)
    parser.add_argument('--output',required=True,type=Path)
    args=parser.parse_args();data=args.source.read_bytes();result=build_manifest(strict_json(data),data)
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(result,indent=2,allow_nan=False)+'\n',encoding='utf-8')
    print('Generated provisional37-arm/1490-key/72-job manifest; source freeze required before execution')

if __name__=='__main__':main()
