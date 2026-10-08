"""Regression for consistent extra top margin in every exported PNG."""
import json
from pathlib import Path
import sys
import tempfile
import unittest
from PIL import Image, ImageChops
sys.path.insert(0, str(Path('../figure-prep').resolve()))
import plot_p4
from test_plot_p4 import synthetic_analysis


class TitleMarginTests(unittest.TestCase):
    def test_all_four_titles_have_48_pixel_blank_top_margin(self):
        with tempfile.TemporaryDirectory() as temporary:
            out=Path(temporary)/'figures'
            plot_p4.export(json.dumps(synthetic_analysis()).encode(), out, synthetic=True)
            for png in out.glob('*.png'):
                image=Image.open(png).convert('RGB')
                top=image.crop((0,0,image.width,48))
                self.assertIsNone(ImageChops.difference(top,Image.new('RGB',top.size,'white')).getbbox(),png.name)


if __name__=='__main__':
    unittest.main()
