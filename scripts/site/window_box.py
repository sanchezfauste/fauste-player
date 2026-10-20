#!/usr/bin/env python3
"""The box of the window that covers a point of a screenshot.

Usage: window_box.py <image> <x> <y> <radius> [left top right bottom]

Prints `WxH+X+Y`: the bounding box of the region of the window's fill colour
that holds the point, one pixel wider on each side for the window's border,
then widened by the optional margins and kept inside the image. The fill
colour is the most common colour within `radius` pixels of the point: pick a
point in an empty part of the window, away from its text. A window's
size depends on the language of its text, so the screenshots of the guide
find it on the screen instead of using a fixed rectangle.

Needs ImageMagick's `convert` (to read the pixels); no Python packages.
"""

import subprocess
import sys
from collections import Counter


def pixels(image):
    size = subprocess.run(
        ["identify", "-format", "%w %h", image], capture_output=True, check=True, text=True
    ).stdout.split()
    width, height = int(size[0]), int(size[1])
    raw = subprocess.run(
        ["convert", image, "-alpha", "off", "-depth", "8", "rgb:-"],
        capture_output=True,
        check=True,
    ).stdout
    return width, height, raw


def main():
    if len(sys.argv) not in (5, 9):
        sys.exit(__doc__)
    image, x, y, radius = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
    margins = [int(m) for m in sys.argv[5:9]] or [0, 0, 0, 0]
    width, height, raw = pixels(image)

    def colour(px, py):
        i = (py * width + px) * 3
        return raw[i : i + 3]

    near = Counter(
        bytes(colour(px, py))
        for py in range(max(0, y - radius), min(height, y + radius + 1))
        for px in range(max(0, x - radius), min(width, x + radius + 1))
    )
    fill = near.most_common(1)[0][0]
    # The fill pixel nearest to the point starts the flood.
    seed = min(
        (
            (abs(px - x) + abs(py - y), px, py)
            for py in range(max(0, y - radius), min(height, y + radius + 1))
            for px in range(max(0, x - radius), min(width, x + radius + 1))
            if colour(px, py) == fill
        ),
    )
    seen = bytearray(width * height)
    stack = [(seed[1], seed[2])]
    left, top, right, bottom = width, height, -1, -1
    while stack:
        px, py = stack.pop()
        i = py * width + px
        if seen[i] or colour(px, py) != fill:
            continue
        seen[i] = 1
        left, right = min(left, px), max(right, px)
        top, bottom = min(top, py), max(bottom, py)
        if px > 0:
            stack.append((px - 1, py))
        if px < width - 1:
            stack.append((px + 1, py))
        if py > 0:
            stack.append((px, py - 1))
        if py < height - 1:
            stack.append((px, py + 1))
    left = max(0, left - 1 - margins[0])
    top = max(0, top - 1 - margins[1])
    right = min(width - 1, right + 1 + margins[2])
    bottom = min(height - 1, bottom + 1 + margins[3])
    print(f"{right - left + 1}x{bottom - top + 1}+{left}+{top}")


if __name__ == "__main__":
    main()
