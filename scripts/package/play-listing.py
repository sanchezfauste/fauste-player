#!/usr/bin/env python3
"""The Google Play listing in packaging/android/play (fastlane's layout).

    scripts/package/play-listing.py check
        Checks every language's texts against Play's limits and the images
        against Play's sizes, that every language has the default
        language's screenshot files, that every interface language
        (crates/fp-app/locales) has a listing, and that the user guide's
        Android screenshots (docs/images/android) are the default
        language's. CI runs it on the stores dry run.
    scripts/package/play-listing.py whatsnew <out-dir>
        Writes the release notes (each language's changelogs/default.txt) as
        <out-dir>/whatsnew-<language>, the layout the upload action reads.
"""

import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LISTING = ROOT / "packaging" / "android" / "play"
LOCALES = ROOT / "crates" / "fp-app" / "locales"
GUIDE_SHOTS = ROOT / "docs" / "images" / "android"
DEFAULT_LANGUAGE = "en-US"

# Play Console's limits, in characters.
LIMITS = {
    "title.txt": 30,
    "short_description.txt": 80,
    "full_description.txt": 4000,
    "changelogs/default.txt": 500,
}
# PNG colour types (the IHDR byte): Play wants the icon as a 32-bit PNG
# and the feature graphic and screenshots without alpha.
RGB, RGBA = 2, 6
COLOUR_NAMES = {RGB: "RGB", RGBA: "RGBA"}
COLOUR_KINDS = {RGB: "24-bit PNG", RGBA: "32-bit PNG"}
# Exact sizes and colour types of the default language's images.
IMAGES = {
    "images/icon.png": ((512, 512), RGBA),
    "images/featureGraphic.png": ((1024, 500), RGB),
}
# Phone screenshots: at least two, each side 320 to 3840 px, the long side
# at most twice the short one. A translation has none (Play shows the
# default language's) or the same files, taken in its language.
SCREENSHOTS = "images/phoneScreenshots"
MIN_SCREENSHOTS = 2
SIDE = (320, 3840)


def languages(listing):
    return sorted(p.name for p in listing.iterdir() if p.is_dir())


def text(path):
    return path.read_text(encoding="utf-8").rstrip("\n")


def png_header(path):
    """The (width, height) and colour type of a PNG, or None."""
    with path.open("rb") as f:
        head = f.read(26)
    if len(head) < 26 or head[:8] != b"\x89PNG\r\n\x1a\n" or head[12:16] != b"IHDR":
        return None
    return struct.unpack(">II", head[16:24]), head[25]


def colour_problem(name, colour, want, lang=DEFAULT_LANGUAGE):
    if colour == want:
        return None
    shown = COLOUR_NAMES.get(colour, f"colour type {colour}")
    return f"{lang}: {name} is {shown}, not {COLOUR_NAMES[want]} ({COLOUR_KINDS[want]})"


def screenshots(listing, lang):
    return sorted((listing / lang / SCREENSHOTS).glob("*.png"))


def screenshot_problems(listing, lang):
    problems = []
    for path in screenshots(listing, lang):
        name = f"{SCREENSHOTS}/{path.name}"
        header = png_header(path)
        if header is None or not screenshot_fits(*header[0]):
            shown = "not a PNG" if header is None else f"{header[0][0]} × {header[0][1]}"
            problems.append(f"{lang}: {name} is {shown} (at most 2:1, {SIDE[0]} to {SIDE[1]} px)")
        elif problem := colour_problem(name, header[1], RGB, lang):
            problems.append(problem)
    return problems


def check(listing, locales=None, guide=None):
    """The listing's problems; with <locales> (a folder of locale tags), also
    the interface languages without a listing, and with <guide>, the guide's
    copies of the default language's screenshots that differ."""
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
    for name, (size, colour) in IMAGES.items():
        path = listing / DEFAULT_LANGUAGE / name
        header = png_header(path) if path.is_file() else None
        if not path.is_file():
            problems.append(f"{DEFAULT_LANGUAGE}: missing {name}")
        elif header is None or header[0] != size:
            shown = "not a PNG" if header is None else header[0]
            problems.append(f"{DEFAULT_LANGUAGE}: {name} is {shown}, not {size}")
        elif problem := colour_problem(name, header[1], colour):
            problems.append(problem)
    shots = screenshots(listing, DEFAULT_LANGUAGE)
    if len(shots) < MIN_SCREENSHOTS:
        problems.append(
            f"{DEFAULT_LANGUAGE}: {len(shots)} phone screenshot(s) in {SCREENSHOTS} "
            f"(at least {MIN_SCREENSHOTS})"
        )
    names = [p.name for p in shots]
    for lang in langs:
        problems += screenshot_problems(listing, lang)
        own = [p.name for p in screenshots(listing, lang)]
        # Play would show a language without its own the default
        # language's, in English.
        if lang != DEFAULT_LANGUAGE and own != names:
            problems.append(
                f"{lang}: phone screenshots {', '.join(own) or '(none)'}, not those of "
                f"{DEFAULT_LANGUAGE} ({', '.join(names)})"
            )
    if locales is not None:
        for tag in sorted(p.name for p in locales.iterdir() if p.is_dir()):
            short = tag.split("-")[0]
            if tag not in langs and short not in langs:
                problems.append(f"{tag}: no Google Play listing ({tag} or {short})")
    if guide is not None:
        copies = sorted(p.name for p in guide.glob("*.png"))
        differ = sorted(
            set(copies).symmetric_difference(names)
            | {n for n in set(copies) & set(names)
               if (guide / n).read_bytes() != (listing / DEFAULT_LANGUAGE / SCREENSHOTS / n).read_bytes()}
        )
        if differ:
            problems.append(
                f"the guide's screenshots in {guide.name} are not those of "
                f"{DEFAULT_LANGUAGE}: {', '.join(differ)}"
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
        problems = check(LISTING, locales=LOCALES, guide=GUIDE_SHOTS)
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
