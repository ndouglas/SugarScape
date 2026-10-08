"""Layout-only rerender from unchanged complete projection and full-source binding."""
import hashlib
import json
from pathlib import Path
import sys
sys.path.insert(0,str(Path('../figure-prep').resolve()))
import plot_p4
projection=Path('presentation-analysis.json').read_bytes()
receipt=json.loads(Path('projection-receipt.json').read_text())
binding=receipt['source']
with Path(binding['path']).open('rb') as stream:
    digest=hashlib.file_digest(stream,'sha256').hexdigest()
assert digest==binding['sha256']=='14caedbb8ed105f1b65e3c07dc5aaed3428180b10122531fc4dc54c6ce00cf9b'
old=json.loads(Path('render-attempt2/figures/plot-inventory.json').read_text())
assert hashlib.sha256(projection).hexdigest()==old['presentation_projection_sha256']
result=plot_p4.export(projection,Path('figures'),synthetic=False,source_binding=binding)
for name in ('chart-inputs.json','chart-inputs.csv'):
    assert (Path('figures')/name).read_bytes()==(Path('render-attempt2/figures')/name).read_bytes()
Path('layout-rerender-receipt.json').write_text(json.dumps(dict(classification='layout_only_rerender',
    original_source_sha256=digest,presentation_projection_sha256=hashlib.sha256(projection).hexdigest(),
    chart_json_csv_actual_byte_equal=True,paired_estimates_recalculated=False,
    controller_preview_concern='primary-lifetime title appeared clipped in preview',
    before_pixel_qa='all four PNG files had26 fully white top rows; no file clipping proved',
    change='all four titles moved from default y=.98 to explicit y=.94',
    actual_figures=result['plots']),indent=2)+'\n')
print(json.dumps(dict(plots=result['plots'],chart_json_csv_byte_equal=True,source_sha256=digest)))
