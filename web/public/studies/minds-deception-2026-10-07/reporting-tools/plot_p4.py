"""Export all registered P4 saved summaries; no simulation or statistical inference.

Use actual saved analysis only after the controller confirms complete census,
validated byte-identical reanalysis, and separate authorization for figure work.
"""
import argparse
import copy
import csv
import hashlib
import io
import itertools
import json
import math
import os
from pathlib import Path
import platform
import sys

ROOT = Path(__file__).resolve().parent
os.environ['MPLCONFIGDIR'] = str(ROOT / 'matplotlib-cache')
SEEDS = list(range(20001, 20041))
METRICS = ('thief_transferred', 'owner_ticks_alive')
PLOTS = ('01-primary-food', '02-primary-lifetime',
         '03-secondary-food', '04-secondary-lifetime')


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def canonical(data):
    return (json.dumps(data, indent=2, sort_keys=True, allow_nan=False) + '\n').encode()


def configurations():
    result = {}
    for sender, view, seen, layout, cost, mirrored in itertools.product(
            ('ordinary', 'matched_neutral', 'sham'), ('ambiguous', 'clear'),
            (False, True), ('on_route', 'off_route'), (0.0, 3.0), (False, True)):
        ident = '-'.join((sender.replace('_', '-'), view, 'seen' if seen else 'unseen',
                          layout.replace('_', '-'), f'cost{cost:g}', f'm{int(mirrored)}'))
        result[ident] = dict(sender=sender, view=view, display_seen=seen, layout=layout,
                            effort_cost=cost, mirrored=mirrored)
    return result


def finite(value):
    return type(value) in (int, float) and math.isfinite(value)


def unavailable_map(entries):
    if not isinstance(entries, list):
        raise ValueError('unavailable lineage must be a list')
    result = {}
    for item in entries:
        seed, reason = item['seed'], item['reason']
        if seed not in SEEDS or seed in result or not isinstance(reason, str) or not reason.strip():
            raise ValueError('unavailable lineage requires unique registered seeds and reasons')
        result[seed] = reason
    return result


def chart_inputs(analysis, source_sha256, *, synthetic):
    if synthetic != (analysis.get('fixture_classification') == 'synthetic_fixture'):
        raise ValueError('synthetic fixture and completed-analysis modes cannot be mislabelled')
    if analysis['schema'] != 'minds-deception-measured-v1':
        raise ValueError('unexpected P4 analysis schema')
    census = analysis['census']
    required = dict(planned=3840, attempted=3840, complete=3840, failed=0,
                    invalid=0, partial=0, pending=0, unstarted=0)
    if any(census.get(key) != value for key, value in required.items()):
        raise ValueError('complete 3840-attempt census required; no subset analysis')
    configs = configurations()
    cells = {}
    for cell in analysis['cells']:
        condition = cell['condition']
        ident = condition['id']
        if ident in cells or condition['lab'] != configs.get(ident):
            raise ValueError('cell coverage/configuration mismatch')
        if cell['seeds'] != SEEDS:
            raise ValueError('each cell requires the exact 40 registered seeds')
        if cell['completed_ticks'] != [64] * 40 or len(cell['biological_groups']) != 40:
            raise ValueError('each cell requires 40 completed horizons and biological-group references')
        if len(cell['wall_seconds']) != 40 or any(not finite(v) or v < 0 for v in cell['wall_seconds']):
            raise ValueError('cell timing must be finite and complete')
        unavailable_map(cell['unavailable_lineage'])
        cells[ident] = cell
    if set(cells) != set(configs):
        raise ValueError('96-cell coverage required')
    endpoints = {}
    for endpoint in analysis['endpoints']:
        key = (endpoint['condition'], endpoint['seed'])
        if key in endpoints:
            raise ValueError('duplicate endpoint coverage')
        if not finite(endpoint['owner_ticks_alive']) or not 0 <= endpoint['owner_ticks_alive'] <= 64:
            raise ValueError('finite bounded lifetime endpoint required')
        amount, reason = endpoint['thief_transferred'], endpoint['lineage_unavailable_reason']
        if amount is None:
            if not isinstance(reason, str) or not reason.strip():
                raise ValueError('unavailable endpoint lineage reason required')
        elif not finite(amount) or amount < 0 or reason is not None:
            raise ValueError('finite food endpoint or unavailable lineage reason required')
        endpoints[key] = endpoint
    if set(endpoints) != set(itertools.product(configs, SEEDS)):
        raise ValueError('complete 3840 endpoint coverage required')
    unavailable_count = 0
    for ident, cell in cells.items():
        actual = {seed: endpoints[ident, seed]['lineage_unavailable_reason']
                  for seed in SEEDS if endpoints[ident, seed]['thief_transferred'] is None}
        if unavailable_map(cell['unavailable_lineage']) != actual:
            raise ValueError('cell unavailable lineage disagrees with endpoints')
        unavailable_count += len(actual)
    if census['unavailable_lineage'] != unavailable_count:
        raise ValueError('census unavailable lineage disagrees with endpoint count')
    expected = {}
    for sham, config in configs.items():
        if config['sender'] != 'sham':
            continue
        for primary, sender in ((True, 'matched_neutral'), (False, 'ordinary')):
            matching = dict(config, sender=sender)
            baseline = next(ident for ident, lab in configs.items() if lab == matching)
            for metric in METRICS:
                expected[(f'{sham} minus {baseline}', metric, primary)] = (sham, baseline)
    estimates = analysis['estimates']
    if len(estimates) != 128:
        raise ValueError('all 128 estimates required')
    plots, seen = {name: [] for name in PLOTS}, set()
    for estimate in estimates:
        key = (estimate['id'], estimate['metric'], estimate['primary'])
        if type(estimate['primary']) is not bool or key not in expected or key in seen:
            raise ValueError('estimate coverage requires each registered contrast once')
        seen.add(key)
        if estimate['denominator'] != 40:
            raise ValueError('full 40-seed denominator required for every estimate')
        sham, baseline = expected[key]
        absent = {}
        if estimate['metric'] == 'thief_transferred':
            for seed in SEEDS:
                reasons = [f"{ident}: {endpoints[ident, seed]['lineage_unavailable_reason']}"
                           for ident in (sham, baseline)
                           if endpoints[ident, seed]['thief_transferred'] is None]
                if reasons:
                    absent[seed] = '; '.join(reasons)
        if unavailable_map(estimate['unavailable']) != absent:
            raise ValueError('estimate unavailable lineage must preserve exact paired reasons')
        summary = estimate['summary']
        if absent:
            if summary is not None:
                raise ValueError('unavailable food summary must remain null, never zero/subset')
        else:
            if not isinstance(summary, dict) or summary['n'] != 40:
                raise ValueError('available summary requires full 40 paired seeds')
            interval = summary['ci95']
            if (not finite(summary['mean']) or not isinstance(interval, list) or len(interval) != 2
                    or any(not finite(v) for v in interval)
                    or not interval[0] <= summary['mean'] <= interval[1]):
                raise ValueError('finite ordered paired Student-t 95% interval required')
            signs = [summary[k] for k in ('positive', 'zero', 'negative')]
            if any(type(v) is not int or v < 0 for v in signs) or sum(signs) != 40:
                raise ValueError('all 40 paired sign counts required')
        index = (0 if estimate['primary'] else 2) + (estimate['metric'] == 'owner_ticks_alive')
        row = copy.deepcopy(estimate)
        row.update(sham_condition=sham, baseline_condition=baseline,
                   strata=copy.deepcopy(configs[sham]))
        plots[PLOTS[index]].append(row)
    if seen != set(expected) or any(len(rows) != 32 for rows in plots.values()):
        raise ValueError('all 128 estimate coverage required, 32 strata per plot')
    for rows in plots.values():
        rows.sort(key=lambda row: (row['strata']['mirrored'], row['sham_condition']))
    return dict(schema='p4-chart-inputs-v1', classification='synthetic_fixture' if synthetic
                else 'completed_saved_analysis', analysis_sha256=source_sha256,
                interval_semantics='paired Student-t descriptive 95%; no significance or aggregate verdict',
                pairing='exact registered seeds 20001-20040; orientations separate; no pooling',
                plots=plots, census=copy.deepcopy(census),
                diagnostics={key: copy.deepcopy(analysis[key]) for key in
                             ('cells', 'repeated_endpoints', 'effective_aliases')},
                biological_frames_and_endpoint_lineage='retained in exact analysis.json bound by analysis_sha256')


def export(source_bytes, destination, *, synthetic, source_binding=None):
    analysis = json.loads(source_bytes)
    source_sha = sha256(source_bytes) if source_binding is None else source_binding['sha256']
    if source_binding is not None and analysis.get('presentation_projection') != source_binding:
        raise ValueError('projection binding differs from full-source parse receipt')
    chart = chart_inputs(analysis, source_sha, synthetic=synthetic)
    if source_binding is not None:
        chart['presentation_projection'] = dict(source_binding, projection_sha256=sha256(source_bytes))
    destination = Path(destination)
    destination.mkdir(parents=True, exist_ok=False)
    import matplotlib
    matplotlib.use('Agg')
    import numpy
    from matplotlib import pyplot as plt, font_manager
    font = Path(font_manager.findfont('DejaVu Sans', fallback_to_default=False))
    identity = dict(python=platform.python_version(), executable=sys.executable,
                    matplotlib=matplotlib.__version__, numpy=numpy.__version__, backend='Agg',
                    font='DejaVu Sans', font_sha256=sha256(font.read_bytes()),
                    dpi=160, svg_hashsalt='p4-complete-summary-v1', timestamp_metadata=None)
    (destination / 'chart-inputs.json').write_bytes(canonical(chart))
    stream = io.StringIO(newline='')
    fields = ('plot', 'id', 'metric', 'primary', 'denominator', 'sham_condition',
              'baseline_condition', 'strata', 'summary', 'unavailable')
    writer = csv.DictWriter(stream, fieldnames=fields, lineterminator='\n')
    writer.writeheader()
    for name, rows in chart['plots'].items():
        for row in rows:
            writer.writerow(dict(plot=name, **{key: json.dumps(value, sort_keys=True, allow_nan=False)
                if key in ('strata', 'summary', 'unavailable') else value for key, value in row.items()}))
    (destination / 'chart-inputs.csv').write_text(stream.getvalue(), encoding='utf-8', newline='')
    settings = {'font.family': 'DejaVu Sans', 'font.size': 9, 'svg.fonttype': 'none',
                'svg.hashsalt': identity['svg_hashsalt'], 'axes.spines.top': False,
                'axes.spines.right': False, 'figure.facecolor': 'white'}
    with plt.rc_context(settings):
        for name, rows in chart['plots'].items():
            fig, axes = plt.subplots(1, 2, figsize=(15, 8), sharex=True)
            primary = rows[0]['primary']
            food = rows[0]['metric'] == 'thief_transferred'
            family = 'Primary: Sham minus Matched-neutral' if primary else 'Secondary: Sham minus Ordinary'
            metric = 'Original food first transferred to thief' if food else 'Owner ticks alive at tick start'
            prefix = 'SYNTHETIC FIXTURE — ' if synthetic else ''
            fig.suptitle(f'{prefix}{family}\n{metric}', fontsize=14, y=.94)
            available = [row['summary'] for row in rows if row['summary'] is not None]
            limits = [bound for summary in available for bound in summary['ci95']] + [0.0]
            low, high = min(limits), max(limits)
            padding = max((high - low) * .18, .5)
            for mirrored, ax in zip((False, True), axes):
                panel = [row for row in rows if row['strata']['mirrored'] == mirrored]
                labels = []
                for i, row in enumerate(panel):
                    strata = row['strata']
                    label = (f"{strata['view']} / {'seen' if strata['display_seen'] else 'unseen'} / "
                             f"{strata['layout'].replace('_', '-')} / cost {strata['effort_cost']:g}")
                    labels.append(label)
                    summary = row['summary']
                    if summary is None:
                        ax.text(.98, i, f"unavailable ({len(row['unavailable'])}/40)",
                                transform=ax.get_yaxis_transform(), ha='right', va='center', fontsize=8,
                                bbox=dict(facecolor='white', edgecolor='none', pad=1))
                    else:
                        lo, hi = summary['ci95']
                        ax.plot([lo, hi], [i, i], color='#315a79', linewidth=1.8)
                        ax.plot(summary['mean'], i, 'o', color='#315a79', markersize=4)
                ax.set_yticks(range(16), labels)
                ax.set_ylim(15.7, -.7)
                ax.set_xlim(low - padding, high + padding)
                ax.axvline(0, color='#777777', linewidth=.8, linestyle='--')
                ax.set_title(('Reflected' if mirrored else 'Base') + ' orientation: all 16 strata')
                ax.set_xlabel('Sham minus baseline (food units)' if food else 'Sham minus baseline (ticks)')
                ax.grid(axis='x', alpha=.16)
            fig.text(.5, .02, 'Every row retains registered n=40. Paired Student-t 95% descriptive intervals. '
                     'Orientations separate; no pooling.\nUnavailable food remains null; zero-width intervals remain points. '
                     'No significance or overall verdict.', ha='center', fontsize=9)
            fig.tight_layout(rect=(0, .085, 1, .91), w_pad=3)
            fig.savefig(destination / f'{name}.png', dpi=160, metadata={'Software': 'P4 saved-summary export'})
            fig.savefig(destination / f'{name}.svg', metadata={'Date': None, 'Creator': 'P4 saved-summary export'})
            plt.close(fig)
    files = [dict(path=path.name, bytes=path.stat().st_size, sha256=sha256(path.read_bytes()))
             for path in sorted(destination.iterdir()) if path.is_file()]
    inventory = dict(schema='p4-figure-inventory-v1', classification=chart['classification'],
                     analysis_sha256=chart['analysis_sha256'], renderer=identity,
                     source_script_sha256=sha256(Path(__file__).read_bytes()),
                     estimates=128, primary=64, secondary=64, plots=4,
                     plotted_strata_per_plot=32, orientations_pooled=False, files=files)
    if source_binding is not None:
        inventory['source_analysis_identity'] = source_binding
        inventory['presentation_projection_sha256'] = sha256(source_bytes)
    (destination / 'plot-inventory.json').write_bytes(canonical(inventory))
    return inventory


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--analysis', required=True, type=Path)
    parser.add_argument('--out', required=True, type=Path)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--synthetic', action='store_true', help='static DTO fixture only')
    mode.add_argument('--completed-authorized', action='store_true',
                      help='controller-authorized, fully validated completed saved analysis')
    args = parser.parse_args()
    if args.completed_authorized and args.analysis.stat().st_size > 100_000_000:
        sys.path.insert(0, str(ROOT.parent / 'actual-artifacts'))
        from render_actual import render_complete
        result = render_complete(args.analysis, args.out)
    else:
        result = export(args.analysis.read_bytes(), args.out, synthetic=args.synthetic)
    print(json.dumps(dict(classification=result['classification'], plots=result['plots'],
                          estimates=result['estimates'], analysis_sha256=result['analysis_sha256'])))


if __name__ == '__main__':
    main()
