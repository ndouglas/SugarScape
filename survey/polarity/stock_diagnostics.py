"""Summarize read-only initial/terminal stock observations; never change scientific judges."""
import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path


def observation_issue(observation, phase, config, outcome):
    if not isinstance(observation, dict):return 'missing observation'
    required={'available','nonpositive_stocks','stock_count','periods','attempted_period','unavailable_reason'}
    if not required<=observation.keys():return 'incomplete observation'
    if observation['available'] is False:
        if any(observation[k] is not None for k in required-{'available','unavailable_reason'}):return 'unavailable observation contains measured values'
        if not isinstance(observation['unavailable_reason'],str) or not observation['unavailable_reason']:return 'unavailable observation lacks reason'
        if outcome.get('valid') is True:return 'valid session observation unavailable'
        return None
    if observation['available'] is not True:return 'availability must be boolean'
    if observation['unavailable_reason'] is not None:return 'available observation has unavailable reason'
    n,count,p,a=(observation[k] for k in ['nonpositive_stocks','stock_count','periods','attempted_period'])
    if any(type(x) is not int for x in [n,count,p,a]):return 'counts and clocks must be integers'
    provincial=config.get('variant') in ['two_level','overextension']
    denominator=config['width']*config['height'] if phase=='initial' or provincial else outcome.get('sovereign_count')
    if type(denominator) is not int or count!=denominator or count<1 or not 0<=n<=count:return 'count or denominator inconsistent'
    clocks=(0,0) if phase=='initial' else (outcome.get('periods'),outcome.get('attempted_period'))
    if (p,a)!=clocks or not 0<=p<=a<=config['horizon'] or a-p>1:return 'observation clocks inconsistent'
    return None


def summarize(manifest, resolved, records, manifest_bytes, session_bytes=None):
    digest=hashlib.sha256(manifest_bytes).hexdigest()
    registered={a['id']:a for a in manifest['arms']}
    entries=resolved.get('arms',[])
    authority={a.get('id'):a.get('config') for a in entries if isinstance(a,dict)}
    authority_ok=(resolved.get('schema_version')==1 and resolved.get('manifest_sha256')==digest
        and len(authority)==len(entries) and set(authority)==set(registered)
        and all(isinstance(authority[aid],dict) and all(authority[aid].get(k)==v for k,v in arm['config'].items()) for aid,arm in registered.items()))
    integrity=[] if authority_ok else ['missing or inconsistent authoritative complete resolution']
    grouped=defaultdict(list)
    for row in records:grouped[row.get('arm','<missing>')].append(row)
    unknown=sorted(set(grouped)-set(registered))
    if unknown:integrity.append('unregistered records retained: '+','.join(unknown))
    arms=[]
    for aid,arm in registered.items():
        rows=grouped[aid];config=authority.get(aid) if authority_ok else None
        seeds=[row.get('seed') for row in rows];expected=set(range(arm['first_seed'],arm['first_seed']+arm['sessions']))
        arm_integrity=[]
        if any(type(s) is not int for s in seeds) or len(set(str(s) for s in seeds))!=len(seeds) or set(s for s in seeds if type(s) is int)!=expected or len(rows)!=arm['sessions']:
            arm_integrity.append('incomplete, duplicate or unexpected registered seeds')
        if any(row.get('manifest_sha256')!=digest for row in rows):arm_integrity.append('manifest hash mismatch')
        integrity.extend(aid+': '+issue for issue in arm_integrity)
        phase_summaries={}
        for phase in ['initial','terminal']:
            subsets={}
            for population in ['all_sessions','valid_sessions','invalid_sessions']:
                selected=[row for row in rows if population=='all_sessions' or (row.get('outcome',{}).get('valid') is True)==(population=='valid_sessions')]
                observed=[];unavailable=Counter();bad=Counter()
                for row in selected:
                    outcome=row.get('outcome',{});diagnostics=row.get('stock_diagnostics',{});issue=None
                    if config is None:issue='authoritative config unavailable'
                    elif row.get('config')!=config:issue='complete resolved config mismatch'
                    elif row.get('manifest_sha256')!=digest:issue='manifest hash mismatch'
                    elif type(outcome.get('valid')) is not bool:issue='outcome validity unavailable'
                    elif not isinstance(diagnostics,dict) or diagnostics.get('schema_version')!=1:issue='missing or unsupported diagnostic schema'
                    else:
                        unit='primitive_cell_stocks' if config.get('variant') in ['two_level','overextension'] else 'sovereign_capital_stocks'
                        if diagnostics.get('unit')!=unit:issue='stock unit mismatch'
                        else:issue=observation_issue(diagnostics.get(phase),phase,config,outcome)
                    if issue:bad[issue]+=1;continue
                    observation=diagnostics[phase]
                    if observation['available']:observed.append(observation)
                    else:unavailable[observation['unavailable_reason']]+=1
                stocks=sum(o['stock_count'] for o in observed);nonpositive=sum(o['nonpositive_stocks'] for o in observed);any_sessions=sum(o['nonpositive_stocks']>0 for o in observed)
                blocked=bool(arm_integrity or not authority_ok or unknown)
                subsets[population]={'received_sessions':len(selected),'available_sessions':len(observed),'unavailable_sessions':sum(unavailable.values()),'invalid_telemetry_sessions':sum(bad.values()),'nonpositive_stocks':nonpositive,'stock_count':stocks,'stock_frequency':nonpositive/stocks if stocks and not blocked else None,'sessions_with_any':any_sessions,'session_frequency':any_sessions/len(observed) if observed and not blocked else None,'unavailable_reasons':dict(unavailable),'invalid_telemetry_reasons':dict(bad),'frequency_condition':'available clock-matched observations; integrity failures block frequency interpretation'}
            phase_summaries[phase]=subsets
        arms.append({'id':aid,'family':arm['family'],'resource_policy':config.get('resource_policy') if config else None,'unit':('primitive_cell_stocks' if config.get('variant') in ['two_level','overextension'] else 'sovereign_capital_stocks') if config else None,'received_sessions':len(rows),'registered_sessions':arm['sessions'],'integrity_issues':arm_integrity,**phase_summaries})
    pools=[]
    keys=sorted({(a['family'],str(a['resource_policy']),str(a['unit'])) for a in arms})
    for family,policy,unit in keys:
        members=[a for a in arms if (a['family'],str(a['resource_policy']),str(a['unit']))==(family,policy,unit)]
        pooled={'family':family,'resource_policy':policy,'unit':unit,'arms':[a['id'] for a in members]}
        blocked=bool(integrity or any(a['integrity_issues'] for a in members))
        for phase in ['initial','terminal']:
            pooled[phase]={}
            for population in ['all_sessions','valid_sessions','invalid_sessions']:
                groups=[a[phase][population] for a in members]
                counts={k:sum(g[k] for g in groups) for k in ['received_sessions','available_sessions','unavailable_sessions','invalid_telemetry_sessions','nonpositive_stocks','stock_count','sessions_with_any']}
                stocks=counts['stock_count'];sessions=counts['available_sessions']
                counts['stock_frequency']=counts['nonpositive_stocks']/stocks if stocks and not blocked else None
                counts['session_frequency']=counts['sessions_with_any']/sessions if sessions and not blocked else None
                counts['unavailable_reasons']=dict(sum((Counter(g['unavailable_reasons']) for g in groups),Counter()))
                counts['invalid_telemetry_reasons']=dict(sum((Counter(g['invalid_telemetry_reasons']) for g in groups),Counter()))
                pooled[phase][population]=counts
        pools.append(pooled)
    return {'schema_version':1,'manifest_sha256':digest,'resolved_config_sha256':hashlib.sha256(json.dumps(resolved,sort_keys=True).encode()).hexdigest(),'session_file_sha256':hashlib.sha256(session_bytes).hexdigest() if session_bytes is not None else None,'raw_record_count':len(records),'unknown_arms':unknown,'integrity_issues':integrity,'measure':'direct initial and terminal nonpositive-stock observations; terminal includes invalid end state when available','stock_frequency_denominator':'sum of observed sovereign-capital stocks for EPM; all primitive-cell stocks for provincial variants','session_frequency_denominator':'number of sessions with available validated observation; invalid outcomes separated; unavailable is never measured zero','period_exposure_frequency':'not measured; no inference from terminal snapshots','pools_by_family_policy_unit':pools,'arms':arms}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['manifest','resolved','sessions','output']:parser.add_argument('--'+name,required=True,type=Path)
    args=parser.parse_args();manifest_bytes=args.manifest.read_bytes();session_bytes=args.sessions.read_bytes()
    records=[json.loads(line) for line in session_bytes.splitlines() if line.strip()]
    result=summarize(json.loads(manifest_bytes),json.loads(args.resolved.read_text()),records,manifest_bytes,session_bytes)
    args.output.write_text(json.dumps(result,indent=2,sort_keys=True,allow_nan=False)+'\n')
    print(f"Retained {len(records)} records; summarized {len(result['arms'])} arms into {args.output}")


if __name__=='__main__':main()
