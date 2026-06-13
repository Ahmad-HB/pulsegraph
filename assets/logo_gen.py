#!/usr/bin/env python3
"""Generate the PulseGraph logo: a dark rounded tile with a 5x5 heatmap grid
whose bright green cells form a rising anti-diagonal band. Emits a 1024px PNG
(icon source) and a scalable SVG (README/repo logo)."""
from PIL import Image, ImageDraw

SIZE = 1024
TILE_RADIUS = 205          # squircle-ish corner radius
PAD = 150                  # padding from tile edge to grid
N = 5                      # 5x5 grid
GAP_RATIO = 0.30           # gap as fraction of cell size
CELL_RADIUS_RATIO = 0.22

TILE_BG = (13, 17, 23, 255)        # #0d1117
RAMP = {
    0: (33, 38, 45, 255),          # #21262d  empty cell (reads against tile)
    2: (38, 166, 65, 255),         # #26a641  mid green
    3: (57, 211, 83, 255),         # #39d353  bright green
}
# Intensity matrix (rows top->bottom) — bright band on the anti-diagonal.
LEVELS = [
    [0, 0, 0, 0, 2],
    [0, 0, 0, 2, 3],
    [0, 0, 2, 3, 3],
    [0, 2, 3, 3, 2],
    [2, 3, 3, 2, 0],
]

grid_area = SIZE - 2 * PAD
# 5*cell + 4*gap = grid_area ; gap = GAP_RATIO*cell
cell = grid_area / (N + (N - 1) * GAP_RATIO)
gap = cell * GAP_RATIO
cell_r = cell * CELL_RADIUS_RATIO


def draw_png():
    img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rounded_rectangle([0, 0, SIZE - 1, SIZE - 1], radius=TILE_RADIUS, fill=TILE_BG)
    for r in range(N):
        for c in range(N):
            x0 = PAD + c * (cell + gap)
            y0 = PAD + r * (cell + gap)
            d.rounded_rectangle(
                [x0, y0, x0 + cell, y0 + cell],
                radius=cell_r,
                fill=RAMP[LEVELS[r][c]],
            )
    img.save("/tmp/pg_logo_source.png")
    print("PNG written: /tmp/pg_logo_source.png")


def hexc(rgba):
    return "#%02x%02x%02x" % rgba[:3]


def draw_svg():
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{SIZE}" height="{SIZE}" '
        f'viewBox="0 0 {SIZE} {SIZE}">',
        f'<rect x="0" y="0" width="{SIZE}" height="{SIZE}" rx="{TILE_RADIUS}" '
        f'ry="{TILE_RADIUS}" fill="{hexc(TILE_BG)}"/>',
    ]
    for r in range(N):
        for c in range(N):
            x0 = PAD + c * (cell + gap)
            y0 = PAD + r * (cell + gap)
            parts.append(
                f'<rect x="{x0:.1f}" y="{y0:.1f}" width="{cell:.1f}" '
                f'height="{cell:.1f}" rx="{cell_r:.1f}" ry="{cell_r:.1f}" '
                f'fill="{hexc(RAMP[LEVELS[r][c]])}"/>'
            )
    parts.append("</svg>\n")
    with open("/tmp/pg_logo.svg", "w") as f:
        f.write("\n".join(parts))
    print("SVG written: /tmp/pg_logo.svg")


draw_png()
draw_svg()
