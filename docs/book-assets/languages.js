// An English-only stub, so a plain `mdbook build docs` or `mdbook serve docs`
// works (without the language menu). scripts/site/guide.py overwrites this
// file in its staging copy with the real data for every book.
window.faustePlayerGuide = {
    current: "en",
    label: "Language",
    notice: null,
    languages: [{ code: "en", name: "English", pages: [] }]
};
