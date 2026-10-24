// The language choice shared by the whole website: the user guide
// (language-menu.js) and the landing page (site/landing-language.js). One
// storage key, so a language chosen anywhere is kept everywhere, and one
// decision for an English page: stay, or go to the visitor's language.
// Tested by scripts/site/tests/site-language.test.js. Uses no external
// resources.
(function (root) {
    "use strict";

    var STORE = "fauste-player-guide-language";

    function stored() {
        try {
            return root.localStorage.getItem(STORE);
        } catch (e) {
            return null;
        }
    }

    function remember(code) {
        try {
            root.localStorage.setItem(STORE, code);
        } catch (e) {
            // Private window or blocked storage: the ?lang=en flag still works.
        }
    }

    // What to do on a page in `current`, given the translation codes
    // (English excluded), the URL's query, the stored choice and the
    // browser's languages: { go: code to open or null, remember: code to
    // store or null }. Only English pages move; a choice (stored, or
    // ?lang=en) wins over the browser, and an unknown stored code counts as
    // no choice.
    function decide(o) {
        var stay = { go: null, remember: null };
        if (o.current !== "en") {
            return stay;
        }
        if (/[?&]lang=en(&|$)/.test(o.search || "")) {
            return { go: null, remember: "en" };
        }
        if (o.stored === "en") {
            return stay;
        }
        if (o.codes.indexOf(o.stored) >= 0) {
            return { go: o.stored, remember: null };
        }
        var wanted = o.wanted || [];
        for (var i = 0; i < wanted.length; i++) {
            var primary = String(wanted[i]).toLowerCase().split(/[-_]/)[0];
            if (primary === "en") {
                return stay;
            }
            if (o.codes.indexOf(primary) >= 0) {
                return { go: primary, remember: null };
            }
        }
        return stay;
    }

    var api = { STORE: STORE, stored: stored, remember: remember, decide: decide };
    if (typeof module !== "undefined" && module.exports) {
        module.exports = api;
    } else {
        root.faustePlayerLanguage = api;
    }
})(typeof window !== "undefined" ? window : this);
