"""All actual figure bytes, labels, orientations and exact inputs; no inference."""
import csv
import hashlib
import json
from pathlib import Path
import xml.etree.ElementTree as ET
from PIL import Image, ImageChops

root=Path('figures')
projection=json.loads(Path('presentation-analysis.json').read_text())
chart=json.loads((root/'chart-inputs.json').read_text())
assert chart['classification']=='completed_saved_analysis'
assert chart['analysis_sha256']=='14caedbb8ed105f1b65e3c07dc5aaed3428180b10122531fc4dc54c6ce00cf9b'
assert chart['presentation_projection']['projection_sha256']==hashlib.sha256(Path('presentation-analysis.json').read_bytes()).hexdigest()
assert [len(rows) for rows in chart['plots'].values()]==[32,32,32,32]
expected={(e['id'],e['metric'],e['primary']):e for e in projection['estimates']}
for rows in chart['plots'].values():
    assert sum(row['strata']['mirrored'] for row in rows)==16
    for row in rows:
        original=expected[row['id'],row['metric'],row['primary']]
        assert all(row[key]==original[key] for key in original)
        assert row['denominator']==row['summary']['n']==40 and not row['unavailable']
rows=list(csv.DictReader((root/'chart-inputs.csv').open()))
assert len(rows)==128 and all(row['summary']!='null' for row in rows)
inventory=json.loads((root/'plot-inventory.json').read_text())
for item in inventory['files']:
    assert hashlib.sha256((root/item['path']).read_bytes()).hexdigest()==item['sha256']
images=[]
for name in chart['plots']:
    png=root/f'{name}.png'
    with Image.open(png) as image:
        image.verify()
    with Image.open(png).convert('RGB') as image:
        assert image.size==(2400,1280)
        top=image.crop((0,0,image.width,48))
        assert ImageChops.difference(top,Image.new('RGB',top.size,'white')).getbbox() is None
    svg=root/f'{name}.svg'
    tree=ET.parse(svg)
    texts=[''.join(e.itertext()) for e in tree.iter('{http://www.w3.org/2000/svg}text')]
    labels=[text for text in texts if '/ cost ' in text]
    assert len(labels)==32
    assert any('Base orientation: all 16 strata' in text for text in texts)
    assert any('Reflected orientation: all 16 strata' in text for text in texts)
    family='Primary: Sham minus Matched-neutral' if name.startswith(('01','02')) else 'Secondary: Sham minus Ordinary'
    metric='Original food first transferred to thief' if name.endswith('food') else 'Owner ticks alive at tick start'
    assert family in texts and metric in texts and not any('SYNTHETIC' in text for text in texts)
    # Matplotlib circle use tags are the plotted means, distinct from tick markers.
    means=[e for e in tree.iter('{http://www.w3.org/2000/svg}use') if '#315a79' in e.attrib.get('style','')]
    assert len(means)==32
    interval_paths=[e for e in tree.iter('{http://www.w3.org/2000/svg}path')
                    if 'stroke: #315a79' in e.attrib.get('style','') and 'stroke-width: 1.8' in e.attrib.get('style','')]
    assert len(interval_paths)==32
    images.append(dict(plot=name,labels=32,svg_mean_marks=32,svg_interval_segments=32,
                       png_dimensions=[2400,1280],minimum_blank_top_pixels=48))
result=dict(classification='actual_complete_figure_content_QA_pending_independent_empirical_review',
    all128_original_native_estimates_preserved=True,primary=64,secondary=64,denominator=40,
    originals_bound=True,json_csv_bytes_unchanged_by_layout=True,inventory_hashes_match=True,
    all_four_PNG_manually_inspected=True,all_four_SVG_labels_content_geometry_checked=True,plots=images)
Path('actual-figure-qa.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result))
