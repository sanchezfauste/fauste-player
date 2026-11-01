#!/usr/bin/env python3
"""The Google Play listing in packaging/android/play (fastlane's layout).

    scripts/package/play-listing.py check
        Checks every language's texts against Play's limits and the images
        against Play's sizes. CI runs it on the stores dry run.
    scripts/package/play-listing.py whatsnew <out-dir>
        Writes the release notes (each language's changelogs/default.txt) as
        <out-dir>/whatsnew-<language>, the layout the upload action reads.
"""

import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LISTING = ROOT / "packaging" / "android" / "play"
DEFAULT_LANGUAGE = "en-US"

# Play Console's limits, in characters.
LIMITS = {
    "title.txt": 30,
    "short_description.txt": 80,
    "full_description.txt": 4000,
    "changelogs/default.txt": 500,
}
# Exact sizes of the default language's images.
IMAGES = {
    "images/icon.png": (512, 512),
    "images/featureGraphic.png": (1024, 500),
}
# Phone screenshots: at least two, each side 320 to 3840 px, the long side
# at most twice the short one.
SCREENSHOTS = "images/phoneScreenshots"
MIN_SCREENSHOTS = 2
SIDE = (320, 3840)


def languages(listing):
    return sorted(p.name for p in listing.iterdir() if p.is_dir())


def text(path):
    return path.read_text(encoding="utf-8").rstrip("\n")


def png_size(path):
    head = path.read_bytes()[:24]
    if head[:8] != b"\x89PNG\r\n\x1a\n" or head[12:16] != b"IHDR":
        return None
    return struct.unpack(">II", head[16:24])


def check(listing):
    problems = []
    langs = languages(listing)
    if DEFAULT_LANGUAGE not in langs:
        problems.append(f"no {DEFAULT_LANGUAGE} listing")
    for lang in langs:
        for name, limit in LIMITS.items():
            path = listing / lang / name
            if not path.is_file():
                problems.append(f"{lang}: missing {name}")
                continue
            value = text(path)
            if not value.strip():
                problems.append(f"{lang}: {name} is empty")
            elif len(value) > limit:
                problems.append(f"{lang}: {name} has {len(value)} characters (at most {limit})")
    for name, size in IMAGES.items():
        path = listing / DEFAULT_LANGUAGE / name
        if not path.is_file():
            problems.append(f"{DEFAULT_LANGUAGE}: missing {name}")
        elif png_size(path) != size:
            problems.append(f"{DEFAULT_LANGUAGE}: {name} is {png_size(path)}, not {size}")
    shots = sorted((listing / DEFAULT_LANGUAGE / SCREENSHOTS).glob("*.png"))
    if len(shots) < MIN_SCREENSHOTS:
        problems.append(
            f"{DEFAULT_LANGUAGE}: {len(shots)} phone screenshot(s) in {SCREENSHOTS} "
            f"(at least {MIN_SCREENSHOTS})"
        )
    for path in shots:
        size = png_size(path)
        if size is None or not screenshot_fits(*size):
            shown = "not a PNG" if size is None else f"{size[0]} × {size[1]}"
            problems.append(
                f"{DEFAULT_LANGUAGE}: {SCREENSHOTS}/{path.name} is {shown} "
                f"(at most 2:1, {SIDE[0]} to {SIDE[1]} px)"
            )
    return problems


def screenshot_fits(width, height):
    short, long = sorted((width, height))
    return SIDE[0] <= short and long <= SIDE[1] and long <= 2 * short


def whatsnew(listing, out):
    out.mkdir(parents=True, exist_ok=True)
    for lang in languages(listing):
        notes = listing / lang / "changelogs" / "default.txt"
        if notes.is_file():
            (out / f"whatsnew-{lang}").write_text(text(notes) + "\n", encoding="utf-8")


def main(argv):
    if argv[1:] == ["check"]:
        problems = check(LISTING)
        for problem in problems:
            print(f"error: {problem}", file=sys.stderr)
        if problems:
            return 1
        print(f"Google Play listing OK: {', '.join(languages(LISTING))}")
        return 0
    if len(argv) == 3 and argv[1] == "whatsnew":
        whatsnew(LISTING, Path(argv[2]))
        return 0
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
