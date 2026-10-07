"""Traceable scientific figures; bands retain source and replication meanings."""
from pathlib import Path
import json
import platform
import hashlib
import math
import textwrap
import numpy as np
import matplotlib
matplotlib.use('Agg')
from matplotlib import pyplot as plt, font_manager
from . import followup, source, records
from .methods import canonical_bytes
from .followup_analysis import census,normalized_histories,load_fresh
from .followup_registration import new_outputs

RENDERER_VERSION='3.10.7'
COLORS={'literal_precision':'#bd5825','prose_precision':'#208468'}
MECHANISM_LABELS={'tagging':'Tagging','alliances':'Alliances','collective_security':'Collective security'}


def chart_inputs(table,historical,old_provenance,phases,*,source_table_sha256=None):
    source.validate_table(table)
    if set(phases)-set(followup.PHASES):raise ValueError('unknown chart phase')
    rows=[]
    slots=table['slots']+[{'id':e['id'],'figure':10,'mobile_share':.5,'mechanism':e['mechanism'],
        'initial_democratic_share':0.,'metric':'clustering_ratio','read_status':'undefined_density_zero','interval':None} for e in table['structural_exclusions']]
    for slot in slots:
        suffix=source.arm_suffix(slot['mobile_share'],slot['mechanism'],slot['initial_democratic_share'])
        old=historical.get('original.'+suffix,[]);old_census=census(old,30);metric=slot['metric']
        observations=[{'arm':h.get('arm','original.'+suffix),'seed':h.get('seed'),
            'value':h['metrics'][metric] if h['complete'] else None,'status':h['status']} for h in old]
        row={k:slot[k] for k in ('id','figure','mobile_share','mechanism','initial_democratic_share','metric','read_status')}
        row.update(source_interval=slot['interval'],original_mean=old_census[metric+'_mean'],
            original_census=old_census,original_values=[h['value'] for h in observations],
            original_observations=observations,phases={})
        for phase,value in phases.items():
            fresh=value['histories'].get('precision.'+suffix,[]);summary=census(fresh,100)
            target=value['targets'].get(slot['id']);prediction=None if target is None else target['result']
            row['phases'][phase]={'mean':summary[metric+'_mean'],'census':summary,
                'predictive_interval':None if prediction is None else prediction['predictive_interval'],
                'unavailable_reason':'undefined_density_zero' if target is None else target['unavailable_reason'],
                'observations':[{'arm':h.get('arm',phase+'.precision.'+suffix),'seed':h.get('seed'),
                    'value':h['metrics'][metric] if h['complete'] else None,'status':h['status']} for h in fresh]}
        rows.append(row)
    return {'schema_version':1,'study_protocol':followup.PROTOCOL,
        'classification':'synthetic_fixture' if table['extraction'].get('classification')=='synthetic_fixture' else 'validated_descriptive_chart_inputs',
        'source_table_sha256':source_table_sha256 or followup.sha(canonical_bytes(table)),
        'source_band_semantics':'digitization_envelopes_not_paper_confidence_intervals',
        'predictive_band_semantics':'95_percent_simulated_30_history_replication_not_mean_confidence_interval',
        'clustering_semantics':'conditioned_on_surviving_democracies_density_zero_undefined',
        'datasets':{'historical_literal_original':old_provenance,**{p:v['provenance'] for p,v in phases.items()}},
        'rows':rows,'structural_exclusions':table['structural_exclusions'],
        'reading_differences':'descriptive_no_cross_reading_tests_or_preferred_reading_selection'}


def renderer_identity():
    if matplotlib.__version__!=RENDERER_VERSION or platform.python_version()!='3.13.5' or np.__version__!='2.4.6':
        raise ValueError('report renderer/runtime differs from frozen versions')
    font=Path(font_manager.findfont('DejaVu Sans',fallback_to_default=False))
    return {'matplotlib':matplotlib.__version__,'numpy':np.__version__,'python':platform.python_version(),
        'backend':'Agg','font':'DejaVu Sans','font_sha256':hashlib.sha256(font.read_bytes()).hexdigest(),
        'svg_hashsalt':'democratic-peace-followup-v1','dpi':160,'timestamp_metadata':None}


def finite_or_gap(value):
    if value is None:return math.nan
    if type(value) not in (int,float) or not math.isfinite(value):raise ValueError('nonfinite chart quantity')
    return value


def density_census_table(ax,rows,phase,available,metric):
    """Equal-width columns keep dense coordinates' counts readable in every panel."""
    cells=[['Density']+[f"{r['initial_democratic_share']:g}" for r in rows],
        ['O complete']+[f"{r['original_census']['complete_count']}/30" for r in rows]]
    if metric=='clustering_ratio':
        cells.extend([[label]+[str(r['original_census'][key]) for r in rows] for label,key in
            (('O defined','clustering_defined_count'),('O extinct','extinction_count'))])
    if available:
        cells.append(['F complete']+[f"{r['phases'][phase]['census']['complete_count']}/100" for r in rows])
        if metric=='clustering_ratio':
            cells.extend([[label]+[str(r['phases'][phase]['census'][key]) for r in rows] for label,key in
                (('F defined','clustering_defined_count'),('F extinct','extinction_count'))])
    height=.06*len(cells)
    table=ax.table(cellText=cells,cellLoc='center',colWidths=[.15]+[.85/len(rows)]*len(rows),
        bbox=[0.,-.24-height,1.,height])
    table.auto_set_font_size(False);table.set_fontsize(5.7)
    for (row,column),cell in table.get_celld().items():
        cell.set_edgecolor('#dddddd');cell.set_linewidth(.35)
        if row==0 or column==0:cell.set_facecolor('#f2f3f4')
    return table


def render(chart,output_dir,*,protected=()):
    identity=renderer_identity();out=Path(output_dir)
    outputs=[out/'chart-inputs.json',out/'plot-inventory.json']+[out/f'fig{f}.{ext}' for f in (9,10,11) for ext in ('svg','png')]
    new_outputs(protected,outputs)
    out.mkdir(parents=True,exist_ok=True)
    chart_bytes=canonical_bytes(chart)+b'\n';outputs[0].write_bytes(chart_bytes)
    settings={'font.family':'DejaVu Sans','font.size':9,'svg.fonttype':'none','svg.hashsalt':identity['svg_hashsalt'],
        'axes.spines.top':False,'axes.spines.right':False,'axes.titleweight':'bold','figure.facecolor':'white',
        'axes.grid':True,'grid.alpha':.15,'path.simplify':False}
    phase_columns=['literal_precision','prose_precision'] if any(p in chart['datasets'] for p in followup.PHASES) else [None]
    with plt.rc_context(settings):
        for figure in (9,10,11):
            fig,axes=plt.subplots(3,len(phase_columns),figsize=(12.4 if len(phase_columns)==2 else 8.4,13.8),squeeze=False)
            mobile=.85 if figure==11 else .5
            metric='clustering_ratio' if figure==10 else 'democratic_share'
            for mi,mechanism in enumerate(followup.baseline.MECHANISMS):
                rows=sorted([r for r in chart['rows'] if r['figure']==figure and r['mechanism']==mechanism],key=lambda r:r['initial_democratic_share'])
                x=[r['initial_democratic_share'] for r in rows]
                for ci,phase in enumerate(phase_columns):
                    ax=axes[mi,ci]
                    lo=[finite_or_gap(None if r['source_interval'] is None else r['source_interval'][0]) for r in rows]
                    hi=[finite_or_gap(None if r['source_interval'] is None else r['source_interval'][1]) for r in rows]
                    ax.fill_between(x,lo,hi,color='#65676b',alpha=.25,label='Source digitization envelope')
                    for r in rows:
                        points=[finite_or_gap(v) for v in r['original_values']]
                        spread=np.linspace(-.005,.005,len(points)) if points else []
                        ax.scatter(np.asarray(spread)+r['initial_democratic_share'],points,s=7,color='#42658b',alpha=.18,zorder=2)
                    ax.plot(x,[finite_or_gap(r['original_mean']) for r in rows],color='#42658b',marker='o',markersize=3,linewidth=1.4,label='Original literal mean (N=30)')
                    available=phase is not None and phase in chart['datasets']
                    if available:
                        entries=[r['phases'][phase] for r in rows];color=COLORS[phase]
                        ax.plot(x,[finite_or_gap(e['mean']) for e in entries],color=color,marker='s',markersize=3,linewidth=1.7,label='Fresh phase mean (N=100)')
                        lower=[finite_or_gap(None if e['predictive_interval'] is None else e['predictive_interval'][0]) for e in entries]
                        upper=[finite_or_gap(None if e['predictive_interval'] is None else e['predictive_interval'][1]) for e in entries]
                        ax.fill_between(x,lower,upper,color=color,alpha=.15,label='95% predictive interval: 30-history replication')
                    label='Original literal descriptive comparison' if phase is None else ('Literal precision' if phase=='literal_precision' else 'Prose precision')+(' — pending' if not available else '')
                    ax.set_title(MECHANISM_LABELS[mechanism]+' · '+label,fontsize=10,loc='left')
                    ax.set_xlim(-.015,1.015)
                    if metric=='democratic_share':ax.set_ylim(-.025,1.025)
                    else:ax.set_ylim(bottom=0.)
                    ax.set_ylabel('Final democratic territory share' if metric=='democratic_share' else 'Survivor-conditioned clustering ratio')
                    ax.set_xlabel('Initial democratic density')
                    ax.set_xticks(x,[f'{v:g}' for v in x],fontsize=6.)
                    density_census_table(ax,rows,phase,available,metric)
                    ax.legend(loc='best',fontsize=6.3,framealpha=.85)
            fig.suptitle(f'Figure {figure}: source envelopes and reconstruction · mobile share {mobile:g}',fontsize=14,x=.08,ha='left')
            footer='Points: individual original observations. Census: complete/required; O=original, F=fresh; defined clustering and extinction counts. Source bands: digitization envelopes, not paper confidence intervals. '
            footer+='Predictive bands: simulated 30-history replications, not confidence intervals for source or N=100 means. Missing/unreadable source points and undefined clustering retain gaps.'
            footer='\n'.join(textwrap.wrap(footer,width=158 if len(phase_columns)==2 else 108))
            if chart['classification']=='synthetic_fixture':footer+='\nSYNTHETIC TEST FIXTURE — NO EMPIRICAL FINDINGS.'
            fig.text(.08,.025,footer,fontsize=7,va='bottom')
            fig.subplots_adjust(left=.09,right=.97,top=.91,bottom=.18,hspace=.8,wspace=.3)
            fig.savefig(out/f'fig{figure}.svg',metadata={'Date':None,'Creator':'SugarScape democratic-peace followup Matplotlib3.10.7'})
            fig.savefig(out/f'fig{figure}.png',dpi=160,metadata={'Software':'SugarScape democratic-peace followup Matplotlib3.10.7'})
            plt.close(fig)
    inventory={'schema_version':1,'classification':chart['classification'],'renderer':identity,
        'chart_inputs_sha256':followup.sha(chart_bytes),'files':[]}
    # Include allsix figureexports without relying on output-list slicing order.
    inventory['files']=[{'path':p.name,'bytes':p.stat().st_size,'sha256':followup.sha(p.read_bytes())} for p in outputs if p.name!='plot-inventory.json']
    (out/'plot-inventory.json').write_bytes(canonical_bytes(inventory)+b'\n')
    return inventory


def main():
    import argparse
    from .historical import historical_dataset
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('source','historical-study-root','historical-inventory','historical-source-archive','historical-binary','output-dir'):
        parser.add_argument('--'+name,required=True,type=Path)
    parser.add_argument('--phase-inputs',type=Path,help='JSON array of explicit phase manifest/source/sessions/resolved/receipt/binary/source_root/findings paths')
    args=parser.parse_args();source_bytes=args.source.read_bytes();table=records.strict_json(source_bytes)
    if followup.sha(source_bytes)!=followup.SOURCE_TABLE_SHA256:raise ValueError('frozen source table identity mismatch')
    protected=[args.source,args.historical_inventory,args.historical_source_archive,args.historical_binary,*args.historical_study_root.glob('*')]
    # Reserve all derived files before reading or validating large datasets.
    outputs=[args.output_dir/name for name in ('chart-inputs.json','plot-inventory.json',*[f'fig{f}.{ext}' for f in (9,10,11) for ext in ('svg','png')])]
    descriptors=[]
    if args.phase_inputs is not None:
        protected.append(args.phase_inputs);descriptors=records.strict_json(args.phase_inputs.read_bytes())
        if not isinstance(descriptors,list) or len(descriptors)>2:raise ValueError('at most two independent phase descriptors')
        fields={'manifest','source','sessions','resolved','receipt','binary','source_root','findings'}
        for d in descriptors:
            records._fields(d,fields,'phase plot inputs')
            if any(not isinstance(v,str) or not v for v in d.values()):raise ValueError('explicit phase paths required')
            protected.extend(Path(v) for k,v in d.items() if k!='source_root')
            value=records.strict_json(Path(d['manifest']).read_bytes())
            protected.extend(Path(d['source_root'])/e['path'] for e in value.get('source_inventory') or [])
    new_outputs(protected,outputs)
    old=historical_dataset(study_root=args.historical_study_root,inventory_path=args.historical_inventory,
        source_archive=args.historical_source_archive,binary=args.historical_binary)
    histories,_=normalized_histories(old['manifest'],old['records'],old['resolved'],old['binding'])
    phases={}
    for d in descriptors:
        value,_,rows,resolved,binding,raw=load_fresh(d['manifest'],d['source'],d['sessions'],d['resolved'],d['receipt'],d['binary'],d['source_root'])
        phase=value['phase']
        if phase in phases:raise ValueError('duplicate plot phase')
        findings=records.strict_json(Path(d['findings']).read_bytes())
        if findings.get('phase')!=phase or findings.get('study_protocol')!=followup.PROTOCOL or findings.get('method_contract_sha256')!=value['method_contract_sha256'] or findings.get('classification')!='registered_offline_followup_findings' or findings.get('provenance')!={'fresh':{'binding':binding,'data_sha256':raw},'historical':{'binding':old['binding'],'data_sha256':old['data_sha256']},'source_table_sha256':followup.SOURCE_TABLE_SHA256}:
            raise ValueError('phase findings/observation provenance mismatch')
        fresh,_=normalized_histories(value,rows,resolved,binding)
        targets=findings['source']['targets']
        if [t['id'] for t in targets]!=[s['id'] for s in table['slots']]:raise ValueError('plot findings source roster mismatch')
        phases[phase]={'histories':fresh,'targets':{t['id']:t for t in targets},
            'provenance':{'binding':binding,'data_sha256':raw,'findings_sha256':followup.sha(Path(d['findings']).read_bytes())}}
    chart=chart_inputs(table,histories,{'binding':old['binding'],'data_sha256':old['data_sha256']},phases,source_table_sha256=followup.sha(source_bytes))
    render(chart,args.output_dir,protected=protected)
    print('Exported Figures9/10/11, chartinputs and deterministic renderer/file identities')


if __name__=='__main__':main()
