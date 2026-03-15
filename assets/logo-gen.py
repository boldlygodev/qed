#!/usr/bin/env python3
“””
qed logo generator — produces a path-based SVG in any colorway.

Usage:
python3 logo-gen.py –bg BG –text TEXT –accent ACCENT [–out OUT]
[–height PX] [–gap FACTOR] [–font PATH]

Example — Catppuccin Mocha:
python3 logo-gen.py   
–bg “#1E1E2E” –text “#CDD6F4” –accent “#CBA6F7”   
–out qed-logo-catppuccin.svg

Example — Dracula:
python3 logo-gen.py   
–bg “#282A36” –text “#F8F8F2” –accent “#FF79C6”   
–out qed-logo-dracula.svg

Requires fontTools:
pip install fonttools
“””

import sys
import argparse
sys.path.insert(0, ‘/usr/local/lib/python3.12/dist-packages’)
from fontTools import ttLib
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.boundsPen import BoundsPen

DEFAULT_FONT = “OverpassMNerdFontMono-Regular.otf”

def extract_char(tt, char):
cmap = tt.getBestCmap()
hmtx = tt[‘hmtx’]
cp = ord(char)
if cp not in cmap:
return None, 0, None
gname = cmap[cp]
pen = SVGPathPen(tt.getGlyphSet())
tt.getGlyphSet()[gname].draw(pen)
adv = hmtx.metrics[gname][0]
bp = BoundsPen(tt.getGlyphSet())
tt.getGlyphSet()[gname].draw(bp)
return pen.getCommands(), adv, bp.bounds

def build_logo(font_path, bg, text, accent,
target_height=80, gap_factor=0.09,
pad_x=32, pad_y=24):
tt    = ttLib.TTFont(font_path)
upm   = tt[‘head’].unitsPerEm
os2   = tt.get(‘OS/2’)
cap_h = getattr(os2, ‘sCapHeight’, 0) or int(upm * 0.72)
asc   = getattr(os2, ‘sTypoAscender’,  int(upm * 0.8))
desc  = abs(getattr(os2, ‘sTypoDescender’, int(upm * 0.2)))
scale = target_height / cap_h
gap   = int(upm * gap_factor)

```
chars  = ['q', '|', 'e', 'd']
colors = [text, accent, text, text]

cmds = []
x = 0
for ch, color in zip(chars, colors):
    d, adv, bounds = extract_char(tt, ch)
    if bounds is None:
        x += adv + gap
        continue
    xMin, yMin, xMax, yMax = bounds
    cmds.append(('path', d, color, x - xMin, asc, scale))
    x += (xMax - xMin) + gap

# Tombstone — constructed square, cap_h × cap_h, sitting on baseline
cmds.append(('rect', x, cap_h, accent, asc, scale))
x += cap_h + gap

ink_w    = (x - gap) * scale
W        = ink_w + pad_x * 2
H        = (asc + desc) * scale + pad_y * 2
baseline = pad_y + asc * scale

out = [
    f'<svg xmlns="http://www.w3.org/2000/svg" '
    f'viewBox="0 0 {W:.2f} {H:.2f}" width="{W:.2f}" height="{H:.2f}">',
    f'  <rect width="{W:.2f}" height="{H:.2f}" fill="{bg}"/>',
]

for cmd in cmds:
    if cmd[0] == 'path':
        _, d, color, tx_fu, _, s = cmd
        tx = tx_fu * s + pad_x
        t  = f"translate({tx:.4f},{baseline:.4f}) scale({s:.7f},{-s:.7f})"
        out.append(f'  <path transform="{t}" fill="{color}" d="{d}"/>')
    elif cmd[0] == 'rect':
        _, x_fu, size_fu, color, _, s = cmd
        rx = x_fu * s + pad_x
        ry = baseline - size_fu * s
        rw = size_fu * s
        rh = size_fu * s
        out.append(f'  <rect x="{rx:.2f}" y="{ry:.2f}" '
                   f'width="{rw:.2f}" height="{rh:.2f}" '
                   f'rx="{rw * 0.04:.2f}" fill="{color}"/>')

out.append('</svg>')
return '\n'.join(out)
```

def main():
p = argparse.ArgumentParser(
description=‘qed logo generator’,
formatter_class=argparse.#!/usr/bin/env python3
“””
qed logo generator — produces a path-based SVG in any colorway.

Usage:
python3 logo-gen.py –bg BG –text TEXT –accent ACCENT [–out OUT]
[–height PX] [–gap FACTOR] [–font PATH]

Example — Catppuccin Mocha:
python3 logo-gen.py   
–bg “#1E1E2E” –text “#CDD6F4” –accent “#CBA6F7”   
–out qed-logo-catppuccin.svg

Requires fontTools:
pip install fonttools
“””

import argparse
from fontTools import ttLib
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.boundsPen import BoundsPen

DEFAULT_FONT = “OverpassMNerdFontMono-Regular.otf”

def extract_char(tt, char):
cmap = tt.getBestCmap()
hmtx = tt[‘hmtx’]
cp = ord(char)
if cp not in cmap:
return None, 0, None
gname = cmap[cp]
pen = SVGPathPen(tt.getGlyphSet())
tt.getGlyphSet()[gname].draw(pen)
adv = hmtx.metrics[gname][0]
bp = BoundsPen(tt.getGlyphSet())
tt.getGlyphSet()[gname].draw(bp)
return pen.getCommands(), adv, bp.bounds

def build_logo(font_path, bg, text, accent,
target_height=80, gap_factor=0.09,
pad_x=32, pad_y=24):
tt    = ttLib.TTFont(font_path)
upm   = tt[‘head’].unitsPerEm
os2   = tt.get(‘OS/2’)
cap_h = getattr(os2, ‘sCapHeight’, 0) or int(upm * 0.72)
asc   = getattr(os2, ‘sTypoAscender’,  int(upm * 0.8))
desc  = abs(getattr(os2, ‘sTypoDescender’, int(upm * 0.2)))
scale = target_height / cap_h
gap   = int(upm * gap_factor)

```
# All five glyphs placed by ink bounds — tombstone treated identically
chars  = ['q', '|', 'e', 'd', '\u220e']   # U+220E END OF PROOF
colors = [text, accent, text, text, accent]

cmds = []
x = 0
for ch, color in zip(chars, colors):
    d, adv, bounds = extract_char(tt, ch)
    if bounds is None:
        x += adv + gap
        continue
    xMin, yMin, xMax, yMax = bounds
    cmds.append((d, color, x - xMin, asc, scale))
    x += (xMax - xMin) + gap

ink_w    = (x - gap) * scale
W        = ink_w + pad_x * 2
H        = (asc + desc) * scale + pad_y * 2
baseline = pad_y + asc * scale

out = [
    '<svg xmlns="http://www.w3.org/2000/svg" '
    f'viewBox="0 0 {W:.2f} {H:.2f}" width="{W:.2f}" height="{H:.2f}">',
    f'  <rect width="{W:.2f}" height="{H:.2f}" fill="{bg}"/>',
]
for d, color, tx_fu, _, s in cmds:
    tx = tx_fu * s + pad_x
    t  = f"translate({tx:.4f},{baseline:.4f}) scale({s:.7f},{-s:.7f})"
    out.append(f'  <path transform="{t}" fill="{color}" d="{d}"/>')
out.append('</svg>')
return '\n'.join(out)
```

def main():
p = argparse.ArgumentParser(description=‘qed logo generator’)
p.add_argument(’–bg’,     required=True,  help=‘Background color (hex)’)
p.add_argument(’–text’,   required=True,  help=‘Text color (hex)’)
p.add_argument(’–accent’, required=True,  help=‘Pipe and tombstone color (hex)’)
p.add_argument(’–out’,    default=‘qed-logo.svg’, help=‘Output SVG path’)
p.add_argument(’–height’, type=int,   default=80,   help=‘Cap height in pixels’)
p.add_argument(’–gap’,    type=float, default=0.09, help=‘Letter gap as fraction of UPM’)
p.add_argument(’–font’,   default=DEFAULT_FONT,     help=‘Path to Overpass Mono OTF/TTF’)
args = p.parse_args()

```
svg = build_logo(args.font, args.bg, args.text, args.accent,
                 target_height=args.height, gap_factor=args.gap)
with open(args.out, 'w') as f:
    f.write(svg)
print(f"Written: {args.out}")
```

if **name** == ‘**main**’:
main(),
epilog=**doc**,
)
p.add_argument(’–bg’,     required=True,  help=‘Background color (hex)’)
p.add_argument(’–text’,   required=True,  help=‘Text color (hex)’)
p.add_argument(’–accent’, required=True,  help=‘Pipe and tombstone color (hex)’)
p.add_argument(’–out’,    default=‘qed-logo.svg’, help=‘Output SVG path’)
p.add_argument(’–height’, type=int,   default=80,   help=‘Cap height in pixels’)
p.add_argument(’–gap’,    type=float, default=0.09, help=‘Letter gap as fraction of UPM’)
p.add_argument(’–font’,   default=DEFAULT_FONT,     help=‘Path to Overpass Mono OTF/TTF’)
args = p.parse_args()

```
svg = build_logo(
    args.font, args.bg, args.text, args.accent,
    target_height=args.height,
    gap_factor=args.gap,
)
with open(args.out, 'w') as f:
    f.write(svg)
print(f"Written: {args.out}")
```

if **name** == ‘**main**’:
main()