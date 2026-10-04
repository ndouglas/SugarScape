"""Reproducible raster evidence and visually audited manual curve envelopes.

Coordinates were read independently from the printed curves and checked against
pixel neighborhoods. This digitizes inferred curve values, not original means.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import subprocess
from .source import PAPER_SHA256, fixture_table, validate_table
from .manifest import DENSITIES

# Full-page pixel coordinates at pdftoppm -scale-to 1800. None means no
# independently attributable trace at that target; no numeric value is imputed.
READINGS = {
    9: {
        'tagging': [892, None, 800, 746, 751, 806, 795, 638, 466, 388, None, 310],
        'alliances': [892, None, 714, 585, 393, 377, 324, 327, 327, 320, None, 310],
        'collective_security': [892, 797, 506, 367, 319, 317, 314, 314, 313, 312, None, 310]},
    10: {
        'tagging': [None, 658, 692, 712, 725, 758, 750, None, None, None, None, 817],
        'alliances': [None, 619, 628, 689, None, None, None, None, None, None, None, 817],
        'collective_security': [None, 490, 552, 639, None, None, None, None, None, None, None, 817]},
    11: {
        'tagging': [793, 777, 763, None, 766, 775, None, None, None, 730, 570, 300],
        'alliances': [793, 767, 735, 704, 673, 666, 641, 550, 439, 373, 320, 300],
        'collective_security': [793, 740, 657, 530, 431, 331, 340, 315, 319, 328, 306, 300]},
}
AXES = {9: {'x': [326, 884], 'y': [892, 310], 'maximum': 1.},
        10: {'x': [329, 798], 'y': [852, 337], 'maximum': 15.},
        11: {'x': [317, 759], 'y': [793, 300], 'maximum': 1.}}
# Wider envelopes at steep/faint/interpolated dashed segments preserve line,
# horizontal calibration and image uncertainty instead of invented precision.
EXTRA_BANDS = {(9, 'collective_security', .1): 22,
    (9, 'collective_security', .15): 12, (9, 'alliances', .15): 32,
    (11, 'collective_security', .1): 22, (11, 'collective_security', .15): 20,
    (11, 'collective_security', .2): 18, (11, 'tagging', .1): 9,
    (11, 'tagging', .2): 9, (11, 'tagging', .25): 9,
    (10, 'collective_security', .1): 20, (10, 'collective_security', .15): 14}


def build_table(image_root, paper):
    image_root = Path(image_root)
    if hashlib.sha256(Path(paper).read_bytes()).hexdigest() != PAPER_SHA256:
        raise ValueError('source PDF SHA256 mismatch')
    table = fixture_table()
    table['extraction'] = {'classification': 'visually_audited_premeasurement',
        'date': '2026-10-03', 'method': 'manual_curve_trace_with_conservative_pixel_envelope',
        'sampling_identity': 'inferred_curve_value_not_raw_original_mean',
        'measurement_used': False,
        'endpoint_policy': 'shared_visually_traceable_endpoints_read_as_curve_values_with_pixel_envelopes',
        'overlap_policy': 'null_target_when_individual_curve_not_separable',
        'missing_policy': 'null_target_when_scan_trace_absent_no_interpolation_imputed'}
    for figure, page in ((9, 17), (10, 19), (11, 20)):
        image = image_root / f'page-{page}.png'
        data = image.read_bytes()
        # PNG IHDR width/height avoids adding a runtime image dependency.
        if data[:8] != b'\x89PNG\r\n\x1a\n':
            raise ValueError('source render is not PNG')
        size = [int.from_bytes(data[16:20], 'big'), int.from_bytes(data[20:24], 'big')]
        if size != [1120, 1800]:
            raise ValueError('source render geometry changed')
        axis = AXES[figure]
        traces = {mechanism: [[axis['x'][0] + density * (axis['x'][1] - axis['x'][0]), y]
            for density, y in zip(DENSITIES, values) if y is not None]
            for mechanism, values in READINGS[figure].items()}
        table['figures'].append({'id': f'fig{figure}', 'printed_page': page + 469,
            'pdf_page_1based': page, 'image_path': f'survey/out/democratic-peace-preparation/page-{page}.png',
            'image_sha256': hashlib.sha256(data).hexdigest(), 'image_size': size,
            'render_command': ['pdftoppm', '-f', str(page), '-l', str(page), '-scale-to', '1800',
                '-png', str(Path(paper).resolve()), 'survey/out/democratic-peace-preparation/page'],
            'axis_landmarks': {'x_pixels': axis['x'], 'x_values': [0., 1.],
                'y_pixels': axis['y'], 'y_values': [0., axis['maximum']]},
            'axis_uncertainty_pixels': 2, 'minimum_line_envelope_pixels': 4,
            'traces': traces,
            'independent_visual_check': {'first_reader_audit': 'full original render plus enlarged chart and local dark-pixel neighborhoods; 2026-10-03', 'second_reader_status': 'pending'},
            'curve_identity': {'tagging': 'thin solid', 'alliances': 'thick dashed',
                'collective_security': 'upper thin solid', 'neutral_response': 'reference diagonal excluded'}
                if figure != 10 else {'tagging': 'lower thin solid before merge',
                    'alliances': 'thick dashed before merge', 'collective_security': 'upper solid before merge',
                    'same_as_initial': 'horizontal reference excluded'}})
    for slot in table['slots']:
        figure = slot['figure']
        density = slot['initial_democratic_share']
        mechanism = slot['mechanism']
        di = DENSITIES.index(density)
        y = READINGS[figure][mechanism][di]
        slot['calibration_id'] = f'fig{figure}'
        slot['attribution'] = 'inferred_curve_value'
        if y is None:
            absent = figure == 11 and mechanism == 'tagging' and density == .15
            slot.update(read_status='absent_source_point' if absent else 'unreadable_overlap',
                interval=None, pixel_reading=None,
                note='Faint scan gap at target; no independently visible point and no interpolated target imputed.'
                    if absent else 'Individual mechanism trace merges with another curve or axis at target; no numeric target imputed.')
            continue
        axis = AXES[figure]
        x = axis['x'][0] + density * (axis['x'][1] - axis['x'][0])
        band = EXTRA_BANDS.get((figure, mechanism, density), 8 if mechanism == 'alliances' else 7)
        ylo, yhi = y - band, y + band
        # Evaluate both ordinate envelopes against all +/-2px axis calibrations.
        candidates = [(bottom - yy) / (bottom - top) * axis['maximum']
            for yy in (ylo, yhi) for bottom in (axis['y'][0] - 2, axis['y'][0] + 2)
            for top in (axis['y'][1] - 2, axis['y'][1] + 2)]
        maximum = 1. if figure != 10 else 1 / density
        slot.update(read_status='readable', interval=[max(0., min(candidates)), min(maximum, max(candidates))],
            pixel_reading={'x_center': x, 'x_envelope': [x - 2, x + 2],
                'y_center': y, 'y_envelope': [ylo, yhi], 'line_and_horizontal_uncertainty_pixels': band},
            note='Shared endpoint visibly traceable to each mechanism; inferred curve value with clamped pixel envelope, not exact raw source mean.'
                if density in (0., 1.) else 'Visually attributed curve ordinate; envelope includes thickness, calibration and steep/dashed/faint trace uncertainty.')
    return validate_table(table)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--paper', required=True, type=Path)
    parser.add_argument('--image-root', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--render', action='store_true')
    args = parser.parse_args()
    if args.render:
        args.image_root.mkdir(parents=True, exist_ok=True)
        for page in (17, 19, 20):
            subprocess.run(['pdftoppm', '-f', str(page), '-l', str(page), '-scale-to', '1800',
                '-png', str(args.paper), str(args.image_root / 'page')], check=True)
    table = build_table(args.image_root, args.paper)
    args.output.write_text(json.dumps(table, indent=2, allow_nan=False) + '\n')
    print(dict(Counter(slot['read_status'] for slot in table['slots'])))


if __name__ == '__main__':
    main()
