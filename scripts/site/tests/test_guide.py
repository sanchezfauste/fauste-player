"""Tests for the Android screenshots of the translated guides in
scripts/site/guide.py (android-images)."""

import importlib.util
import os
import tempfile
import unittest

SCRIPT = os.path.join(os.path.dirname(__file__), "..", "guide.py")
spec = importlib.util.spec_from_file_location("guide", SCRIPT)
guide = importlib.util.module_from_spec(spec)
spec.loader.exec_module(guide)


def write(path, text=""):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(text)


def read(path):
    with open(path, encoding="utf-8") as fh:
        return fh.read()


PAGE = ('<img src="../../images/android/1-live.png">'
        '<img src="../../images/android/2-carts.png">'
        '<img src="../../images/guide/player.png">')


class AndroidImagesTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = self.tmp.name
        self.site = os.path.join(root, "site")
        self.play = os.path.join(root, "play")
        self.i18n = os.path.join(root, "i18n")
        self.locales = os.path.join(root, "locales")
        for tag in ("en-US", "ca-ES", "pt-PT", "de-DE"):
            os.makedirs(os.path.join(self.locales, tag))
        for lang in ("ca", "pt", "de"):
            os.makedirs(os.path.join(self.i18n, lang))
            write(os.path.join(self.site, "guide", lang, "android.html"), PAGE)
        self.saved = guide.PLAY, guide.I18N, guide.LOCALES
        guide.PLAY, guide.I18N, guide.LOCALES = self.play, self.i18n, self.locales

    def tearDown(self):
        guide.PLAY, guide.I18N, guide.LOCALES = self.saved
        self.tmp.cleanup()

    def shot(self, folder, name):
        write(os.path.join(self.play, folder, "images", "phoneScreenshots", name), "png")

    def page(self, lang):
        return read(os.path.join(self.site, "guide", lang, "android.html"))

    def test_a_translation_links_its_own_screenshots_and_keeps_english_for_the_rest(self):
        self.shot("de-DE", "1-live.png")
        guide.android_images(self.site)
        self.assertTrue(os.path.isfile(os.path.join(self.site, "images", "de", "android", "1-live.png")))
        self.assertEqual(
            self.page("de"),
            '<img src="../../images/de/android/1-live.png">'
            '<img src="../../images/android/2-carts.png">'
            '<img src="../../images/guide/player.png">',
        )

    def test_a_translation_without_screenshots_keeps_the_english_ones(self):
        guide.android_images(self.site)
        self.assertEqual(self.page("de"), PAGE)

    def test_the_listing_of_a_language_is_its_own_code_or_its_interface_locale(self):
        os.makedirs(os.path.join(self.play, "ca"))
        os.makedirs(os.path.join(self.play, "pt-BR"))
        os.makedirs(os.path.join(self.play, "pt-PT"))
        self.assertEqual(guide.play_listing("ca"), os.path.join(self.play, "ca"))
        # pt-BR is not an interface locale: pt-PT is pt's listing.
        self.assertEqual(guide.play_listing("pt"), os.path.join(self.play, "pt-PT"))
        self.assertIsNone(guide.play_listing("de"))

    def test_no_listing_folder_changes_nothing(self):
        guide.android_images(self.site)
        self.assertFalse(os.path.isdir(self.play))
        self.assertEqual(self.page("pt"), PAGE)


if __name__ == "__main__":
    unittest.main()
