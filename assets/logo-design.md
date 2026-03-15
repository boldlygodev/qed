# qed Logo

## Design

The logo is a wordmark: **`q|ed∎`**

Every character does double duty.

The **pipe** (`|`) is the literal syntax that separates a selector from a
processor in every `qed` script — `at(pattern) | processor`.
It appears in the wordmark as a slim accent in the brand color, splitting
`q` from `ed` exactly as it splits selection from action in the language.

The **tombstone** (`∎`) is the traditional end-of-proof mark — *quod erat
demonstrandum*, that which was to be demonstrated.
It names the tool and completes it in the same gesture.
A `qed` invocation is a proof: given this input, produce this output.
The tombstone says it is done.

Together the wordmark reads as a `qed` script fragment that terminates itself.

-----

## Typeface

**Overpass Mono Regular** (Nerd Font variant).

Overpass Mono was chosen over heavier coding fonts for its slightly humanist
construction — the strokes have more personality than geometric mono fonts
while remaining unmistakably terminal.
Regular weight reads confidently at logo sizes; the thin strokes give the pipe
character its correct visual weight as an accent rather than a full letter.

The tombstone is the Unicode `∎` glyph (U+220E) taken directly from
Overpass Mono — 392×511 font units, sitting on the baseline and rising to
approximately x-height.
It is placed by its ink bounds, the same way every other character in the
wordmark is placed, so spacing is optically consistent throughout.

-----

## Default colorways

Two canonical colorways are provided.

### Light — terminal green

```
Background  #FFFFFF
Text        #0D1117
Accent      #3FB950
```

File: `qed-logo.svg`

Terminal green (`#3FB950`) is GitHub’s success/diff-add green.
It reads as “new output,” “pass,” “done” — all correct connotations for a
tool that transforms text and exits zero.
On a white background it has sufficient contrast for accessibility while
reading as warm rather than clinical.

### Dark — terminal green

```
Background  #0D1117
Text        #E6EDF3
Accent      #3FB950
```

File: `qed-logo-dark.svg`

The dark variant uses GitHub’s dark canvas color.
Same accent, inverted field.
Use this in dark-mode README contexts, dark documentation sites, or anywhere
the light variant would look out of place.

### README usage

GitHub supports automatic light/dark switching via the `<picture>` element:

```markdown
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="qed-logo-dark.svg">
  <img src="qed-logo.svg" alt="qed" height="80">
</picture>
```

-----

## Terminal theme colorways

These colorways are provided as drop-in variants for users whose terminal
or editor uses a recognized color scheme.
Each uses the scheme’s canonical background and a highlight color native
to that theme.

|Theme           |Background|Text     |Accent   |
|----------------|----------|---------|---------|
|Dracula         |`#282A36` |`#F8F8F2`|`#FF79C6`|
|Nord            |`#2E3440` |`#ECEFF4`|`#88C0D0`|
|Gruvbox         |`#282828` |`#EBDBB2`|`#B8BB26`|
|Solarized Dark  |`#002B36` |`#839496`|`#268BD2`|
|Monokai         |`#272822` |`#F8F8F2`|`#A6E22E`|
|Tokyo Night     |`#1A1B26` |`#C0CAF5`|`#7AA2F7`|
|Catppuccin Mocha|`#1E1E2E` |`#CDD6F4`|`#CBA6F7`|

-----

## Custom colorways

To generate a logo in any colorway, run `logo-gen.py` with three hex colors:

```sh
python3 logo-gen.py \
  --bg    "#1E1E2E" \
  --text  "#CDD6F4" \
  --accent "#CBA6F7" \
  --out   qed-logo-catppuccin.svg
```

### `logo-gen.py`

```python
#!/usr/bin/env python3
"""
qed logo generator — produces a path-based SVG in any colorway.

Usage:
  python3 logo-gen.py --bg BG --text TEXT --accent ACCENT [--out OUT]
                      [--height PX] [--gap FACTOR] [--font PATH]

Arguments:
  --bg       Background fill color (hex, e.g. #FFFFFF)
  --text     Text fill color (hex, e.g. #0D1117)
  --accent   Pipe and tombstone color (hex, e.g. #3FB950)
  --out      Output SVG path (default: qed-logo.svg)
  --height   Cap height in pixels (default: 80)
  --gap      Letter gap as fraction of UPM (default: 0.09)
  --font     Path to Overpass Mono Regular OTF/TTF
"""

import argparse
from fontTools import ttLib
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.boundsPen import BoundsPen

DEFAULT_FONT = "OverpassMNerdFontMono-Regular.otf"


def extract_char(tt, char):
    cmap = tt.getBestCmap()
    hmtx = tt['hmtx']
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
    upm   = tt['head'].unitsPerEm
    os2   = tt.get('OS/2')
    cap_h = getattr(os2, 'sCapHeight', 0) or int(upm * 0.72)
    asc   = getattr(os2, 'sTypoAscender',  int(upm * 0.8))
    desc  = abs(getattr(os2, 'sTypoDescender', int(upm * 0.2)))
    scale = target_height / cap_h
    gap   = int(upm * gap_factor)

    # All five glyphs placed by ink bounds — tombstone treated identically
    chars  = ['q', '|', 'e', 'd', '\u220e']   # ∎ = U+220E
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
        f'<svg xmlns="http://www.w3.org/2000/svg" '
        f'viewBox="0 0 {W:.2f} {H:.2f}" width="{W:.2f}" height="{H:.2f}">',
        f'  <rect width="{W:.2f}" height="{H:.2f}" fill="{bg}"/>',
    ]
    for d, color, tx_fu, _, s in cmds:
        tx = tx_fu * s + pad_x
        t  = f"translate({tx:.4f},{baseline:.4f}) scale({s:.7f},{-s:.7f})"
        out.append(f'  <path transform="{t}" fill="{color}" d="{d}"/>')
    out.append('</svg>')
    return '\n'.join(out)


def main():
    p = argparse.ArgumentParser(description="qed logo generator")
    p.add_argument('--bg',     required=True,  help='Background color (hex)')
    p.add_argument('--text',   required=True,  help='Text color (hex)')
    p.add_argument('--accent', required=True,  help='Pipe and tombstone color (hex)')
    p.add_argument('--out',    default='qed-logo.svg', help='Output SVG path')
    p.add_argument('--height', type=int,   default=80,   help='Cap height in pixels')
    p.add_argument('--gap',    type=float, default=0.09, help='Letter gap as fraction of UPM')
    p.add_argument('--font',   default=DEFAULT_FONT,     help='Path to Overpass Mono OTF/TTF')
    args = p.parse_args()

    svg = build_logo(args.font, args.bg, args.text, args.accent,
                     target_height=args.height, gap_factor=args.gap)
    with open(args.out, 'w') as f:
        f.write(svg)
    print(f"Written: {args.out}")


if __name__ == '__main__':
    main()
```

### Guidelines for custom colorways

**Contrast.** The accent color must be legible against the background.
Aim for a contrast ratio of at least 4.5:1 for the accent against the
background (WCAG AA).

**Accent role.** The accent should read as active or highlighted — a color
your terminal theme uses for selections, diffs, or success states works well.
Avoid neutral grays; the pipe and tombstone lose their visual separation from
the text.

**Text color.** Use the theme’s default foreground, not pure white or pure
black unless the theme does.

**Don’t modify the geometry.** The proportions, gap factor, and tombstone
construction are fixed.
Only the three colors change between colorways.

-----

## Files

|File               |Description                          |
|-------------------|-------------------------------------|
|`qed-logo.svg`     |Canonical light background           |
|`qed-logo-dark.svg`|Canonical dark background            |
|`qed-logo-2x.svg`  |2× resolution for retina / docs sites|
|`logo-gen.py`      |Generator script for custom colorways|

-----

## Font license

Overpass Mono is released under the SIL Open Font License 1.1.
The Nerd Font patches are released under the MIT License.
Both licenses permit use in open source and commercial projects without
restriction, provided the font itself is not sold standalone.