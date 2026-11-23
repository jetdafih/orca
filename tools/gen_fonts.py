#!/usr/bin/env python3
"""Writes assets/orca/font/*.json: one TTF provider per face, tier and GUI scale.

A face is the CSS size from orca.html (10-16) and a weight. Each file's em is the nominal one
(css * tier * 1.12 GUI px) nudged by at most 4% so that, at `oversample` texels per GUI pixel
(one texel = one screen pixel), the capitals, the x-height and the horizontal strokes land close
to whole pixels. Unhinted outlines at these sizes otherwise leave half-covered rows on every edge."""
import json, sys
TIERS = [0.6, 0.5, 0.42, 0.35]
SIZES = [10, 11, 12, 13, 14, 16]
# Outfit metrics per 1000 em: cap height, x-height, horizontal bar of E.
M = {'o': ('outfit.ttf', 0.700, 0.480, 0.105), 'b': ('outfit_bold.ttf', 0.706, 0.486, 0.135)}
frac = lambda v: abs(v - round(v))

def best(w, nominal_px):
    _, cap, xh, bar = M[w]
    cands = [nominal_px * (0.96 + i * 0.001) for i in range(81)]
    def cost(e):
        return 3 * frac(cap * e) + 2 * frac(xh * e) + frac(bar * e) + 10 * abs(e / nominal_px - 1)
    return min(cands, key=cost)

# Sora is ~8% wider than Outfit and a touch taller in the capitals: shrink it to keep the layout.
# `--sora` writes sora.ttf (SemiBold, stems ~2 px) and sora_bold.ttf (Bold) at that size, unsnapped.
SORA = '--sora' in sys.argv
if SORA:
    M = {'o': ('sora.ttf',) + M['o'][1:], 'b': ('sora_bold.ttf',) + M['b'][1:]}
out = sys.argv[1]
for w in 'bo':
    for css in SIZES:
        for t, k in enumerate(TIERS):
            for s in range(1, 7):
                nominal = css * k * 1.12 * s
                em = nominal * 0.92 if SORA else (best(w, nominal) if '--plain' not in sys.argv else nominal)
                size = round(em / s, 4)
                j = {"providers": [{"type": "ttf", "file": "orca:" + M[w][0], "size": size, "oversample": float(s), "shift": [0, 0]}]}
                json.dump(j, open(f"{out}/{w}{css}t{t}s{s}.json", "w"), separators=(',', ':'))
print('ok')
