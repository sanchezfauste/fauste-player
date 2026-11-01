"""Tests for scripts/package/play-listing.py."""

import importlib.util
import struct
import tempfile
import unittest
import zlib
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "play-listing.py"
spec = importlib.util.spec_from_file_location("play_listing", SCRIPT)
play_listing = importlib.util.module_from_spec(spec)
spec.loader.exec_module(play_listing)


def png(path, width, height, colour=2):
    """A minimal PNG header: enough for the size and colour type checks
    (2 is RGB, 6 is RGBA)."""
    ihdr = struct.pack(">IIBBBBB", width, height, 8, colour, 0, 0, 0)
    chunk = b"IHDR" + ihdr
    data = struct.pack(">I", len(ihdr)) + chunk + struct.pack(">I", zlib.crc32(chunk))
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + data)


def language(root, lang, title="Fauste Player", notes="What's new"):
    base = root / lang
    (base / "changelogs").mkdir(parents=True)
    (base / "title.txt").write_text(title + "\n", encoding="utf-8")
    (base / "short_description.txt").write_text("Radio playout\n", encoding="utf-8")
    (base / "full_description.txt").write_text("Radio playout.\n", encoding="utf-8")
    (base / "changelogs" / "default.txt").write_text(notes + "\n", encoding="utf-8")


class CheckTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name) / "listing"
        language(self.root, "en-US")
        png(self.root / "en-US/images/icon.png", 512, 512, colour=6)
        png(self.root / "en-US/images/featureGraphic.png", 1024, 500)
        png(self.root / "en-US/images/phoneScreenshots/1-live.png", 1080, 2160)
        png(self.root / "en-US/images/phoneScreenshots/2-carts.png", 1080, 2160)

    def tearDown(self):
        self.tmp.cleanup()

    def test_a_complete_listing_passes(self):
        language(self.root, "es-ES")
        self.assertEqual(play_listing.check(self.root), [])

    def test_the_limit_counts_characters_not_bytes(self):
        language(self.root, "es-ES", title="Á" * 30)
        self.assertEqual(play_listing.check(self.root), [])

    def test_a_long_title_fails(self):
        language(self.root, "es-ES", title="x" * 31)
        self.assertEqual(
            play_listing.check(self.root),
            ["es-ES: title.txt has 31 characters (at most 30)"],
        )

    def test_a_missing_text_fails(self):
        (self.root / "en-US/short_description.txt").unlink()
        self.assertEqual(play_listing.check(self.root), ["en-US: missing short_description.txt"])

    def test_an_empty_text_fails(self):
        (self.root / "en-US/title.txt").write_text("\n", encoding="utf-8")
        self.assertEqual(play_listing.check(self.root), ["en-US: title.txt is empty"])

    def test_the_default_language_is_required(self):
        language(self.root, "es-ES")
        for path in sorted((self.root / "en-US").rglob("*"), reverse=True):
            path.unlink() if path.is_file() else path.rmdir()
        (self.root / "en-US").rmdir()
        problems = play_listing.check(self.root)
        self.assertIn("no en-US listing", problems)

    def test_a_feature_graphic_of_another_size_fails(self):
        png(self.root / "en-US/images/featureGraphic.png", 1024, 512)
        self.assertEqual(
            play_listing.check(self.root),
            ["en-US: images/featureGraphic.png is (1024, 512), not (1024, 500)"],
        )

    def test_the_icon_is_a_32_bit_png(self):
        png(self.root / "en-US/images/icon.png", 512, 512, colour=2)
        self.assertEqual(
            play_listing.check(self.root),
            ["en-US: images/icon.png is RGB, not RGBA (32-bit PNG)"],
        )

    def test_a_screenshot_with_alpha_fails(self):
        png(self.root / "en-US/images/phoneScreenshots/2-carts.png", 1080, 2160, colour=6)
        self.assertEqual(
            play_listing.check(self.root),
            ["en-US: images/phoneScreenshots/2-carts.png is RGBA, not RGB (24-bit PNG)"],
        )

    def test_a_feature_graphic_with_alpha_fails(self):
        png(self.root / "en-US/images/featureGraphic.png", 1024, 500, colour=6)
        self.assertEqual(
            play_listing.check(self.root),
            ["en-US: images/featureGraphic.png is RGBA, not RGB (24-bit PNG)"],
        )

    def test_a_file_that_is_not_a_png_fails(self):
        (self.root / "en-US/images/phoneScreenshots/2-carts.png").write_bytes(b"JFIF")
        self.assertEqual(
            play_listing.check(self.root),
            ["en-US: images/phoneScreenshots/2-carts.png is not a PNG (at most 2:1, 320 to 3840 px)"],
        )

    def test_two_phone_screenshots_are_required(self):
        (self.root / "en-US/images/phoneScreenshots/2-carts.png").unlink()
        self.assertEqual(
            play_listing.check(self.root),
            ["en-US: 1 phone screenshot(s) in images/phoneScreenshots (at least 2)"],
        )

    def test_a_screenshot_longer_than_twice_its_width_fails(self):
        png(self.root / "en-US/images/phoneScreenshots/2-carts.png", 1080, 2400)
        self.assertEqual(
            play_listing.check(self.root),
            ["en-US: images/phoneScreenshots/2-carts.png is 1080 × 2400 (at most 2:1, 320 to 3840 px)"],
        )

    def test_a_screenshot_too_small_fails(self):
        png(self.root / "en-US/images/phoneScreenshots/2-carts.png", 300, 500)
        self.assertEqual(
            play_listing.check(self.root),
            ["en-US: images/phoneScreenshots/2-carts.png is 300 × 500 (at most 2:1, 320 to 3840 px)"],
        )

    def test_a_translation_without_screenshots_passes(self):
        # Play shows the default language's.
        language(self.root, "es-ES")
        self.assertEqual(play_listing.check(self.root), [])

    def test_a_translation_s_screenshots_are_checked(self):
        language(self.root, "es-ES")
        png(self.root / "es-ES/images/phoneScreenshots/1-live.png", 1080, 2160)
        png(self.root / "es-ES/images/phoneScreenshots/2-carts.png", 1080, 2400)
        self.assertEqual(
            play_listing.check(self.root),
            ["es-ES: images/phoneScreenshots/2-carts.png is 1080 × 2400 (at most 2:1, 320 to 3840 px)"],
        )

    def test_a_translation_has_the_default_language_s_screenshots(self):
        language(self.root, "es-ES")
        png(self.root / "es-ES/images/phoneScreenshots/1-live.png", 1080, 2160)
        png(self.root / "es-ES/images/phoneScreenshots/9-other.png", 1080, 2160)
        self.assertEqual(
            play_listing.check(self.root),
            ["es-ES: phone screenshots 1-live.png, 9-other.png, not those of en-US "
             "(1-live.png, 2-carts.png)"],
        )

    def test_every_interface_locale_has_a_listing(self):
        locales = self.root.parent / "locales"
        for tag in ("en-US", "ca-ES", "es-ES", "de-DE"):
            (locales / tag).mkdir(parents=True, exist_ok=True)
        language(self.root, "ca")
        language(self.root, "es-ES")
        self.assertEqual(
            play_listing.check(self.root, locales=locales),
            ["de-DE: no Google Play listing (de-DE or de)"],
        )

    def test_the_guide_shows_the_default_language_s_screenshots(self):
        guide = self.root.parent / "guide-shots"
        guide.mkdir()
        shots = self.root / "en-US/images/phoneScreenshots"
        (guide / "1-live.png").write_bytes((shots / "1-live.png").read_bytes())
        self.assertEqual(
            play_listing.check(self.root, guide=guide),
            ["the guide's screenshots in guide-shots are not those of en-US: 2-carts.png"],
        )
        png(guide / "2-carts.png", 1080, 1000)
        self.assertEqual(
            play_listing.check(self.root, guide=guide),
            ["the guide's screenshots in guide-shots are not those of en-US: 2-carts.png"],
        )
        (guide / "2-carts.png").write_bytes((shots / "2-carts.png").read_bytes())
        self.assertEqual(play_listing.check(self.root, guide=guide), [])

    def test_the_repository_listing_passes(self):
        self.assertEqual(
            play_listing.check(
                play_listing.LISTING, locales=play_listing.LOCALES, guide=play_listing.GUIDE_SHOTS
            ),
            [],
        )


class WhatsNewTest(unittest.TestCase):
    def test_writes_one_file_per_language(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "listing"
            language(root, "en-US", notes="New")
            language(root, "es-ES", notes="Nuevo")
            out = Path(tmp) / "out"
            play_listing.whatsnew(root, out)
            self.assertEqual(sorted(p.name for p in out.iterdir()), ["whatsnew-en-US", "whatsnew-es-ES"])
            self.assertEqual((out / "whatsnew-es-ES").read_text(encoding="utf-8"), "Nuevo\n")


if __name__ == "__main__":
    unittest.main()
