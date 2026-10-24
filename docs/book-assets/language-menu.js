// The language menu in the guide's top bar, the notice on a translated page
// that it was translated with AI and, on an English page, the jump to the
// visitor's language (the one they chose, else the browser's). The data
// (languages, the pages each one has, the notice) comes from languages.js,
// which scripts/site/guide.py generates for every book; the choice and the
// decision come from site-language.js, shared with the landing page. Uses
// no external resources.
(function () {
    "use strict";
    var data = window.faustePlayerGuide;
    var shared = window.faustePlayerLanguage;
    if (!data || !shared || typeof path_to_root === "undefined") {
        return;
    }

    // The book's root, the guide's root (the English book) and this page's
    // path inside the book.
    var bookRoot = new URL(path_to_root || "./", window.location.href);
    var guideRoot = data.current === "en" ? bookRoot : new URL("../", bookRoot);
    var path = window.location.pathname;
    var page = path.indexOf(bookRoot.pathname) === 0
        ? decodeURIComponent(path.slice(bookRoot.pathname.length)) : "";
    if (page === "" || page.charAt(page.length - 1) === "/") {
        page += "index.html";
    }

    var remember = shared.remember;

    // Choosing a language, in the menu or by the notice's link, is final:
    // the automatic jump never overrides it.
    function chosen(link, lang) {
        link.addEventListener("click", function () {
            remember(lang.code);
        });
    }

    function href(lang) {
        var base = lang.code === "en" ? guideRoot : new URL(lang.code + "/", guideRoot);
        // A page the other language does not have yet opens its start page.
        // Translations keep the English heading ids, so the anchor carries over.
        if (lang.pages.indexOf(page) < 0) {
            return new URL("index.html", base).href;
        }
        var url = new URL(page + window.location.hash, base);
        // A link to English says so, for browsers that cannot store the choice.
        if (lang.code === "en") {
            url.search = "?lang=en";
        }
        return url.href;
    }

    // An English page goes to the visitor's language: the one stored by a
    // choice, else the first of the browser's languages that is English or
    // has a translation (site-language.js decides). ?lang=en stores
    // English. Without JavaScript nothing moves.
    function detect() {
        var choice = shared.decide({
            current: data.current,
            codes: data.languages.slice(1).map(function (l) { return l.code; }),
            search: window.location.search,
            stored: shared.stored(),
            wanted: window.navigator.languages ||
                (window.navigator.language ? [window.navigator.language] : [])
        });
        if (choice.remember) {
            remember(choice.remember);
        }
        for (var j = 1; choice.go && j < data.languages.length; j++) {
            if (data.languages[j].code === choice.go) {
                window.location.replace(href(data.languages[j]));
                return;
            }
        }
    }

    // The links that carry the current #anchor; refreshed when it changes.
    var anchored = [];

    function refresh() {
        anchored.forEach(function (item) {
            item.link.href = href(item.lang);
        });
    }

    function languageMenu() {
        var bar = document.querySelector("#mdbook-menu-bar .right-buttons");
        if (!bar || data.languages.length < 2) {
            return;
        }
        var wrap = document.createElement("div");
        wrap.className = "language-menu";
        var button = document.createElement("button");
        button.type = "button";
        button.className = "icon-button";
        button.id = "language-toggle";
        button.title = data.label;
        button.setAttribute("aria-label", data.label);
        button.setAttribute("aria-haspopup", "true");
        button.setAttribute("aria-expanded", "false");
        button.setAttribute("aria-controls", "language-list");
        button.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" ' +
            'width="1em" height="1em" fill="none" stroke="currentColor" stroke-width="2" ' +
            'aria-hidden="true"><circle cx="12" cy="12" r="10"/><path d="M2 12h20"/>' +
            '<path d="M12 2a15 15 0 0 1 0 20a15 15 0 0 1 0-20z"/></svg>';
        var list = document.createElement("ul");
        list.id = "language-list";
        list.className = "language-popup";
        list.setAttribute("role", "menu");
        list.setAttribute("aria-label", data.label);
        data.languages.forEach(function (lang) {
            var item = document.createElement("li");
            item.setAttribute("role", "none");
            var link = document.createElement("a");
            link.setAttribute("role", "menuitem");
            link.href = href(lang);
            anchored.push({ link: link, lang: lang });
            link.lang = lang.code;
            link.hreflang = lang.code;
            link.textContent = lang.name;
            chosen(link, lang);
            if (lang.code === data.current) {
                link.setAttribute("aria-current", "true");
            }
            item.appendChild(link);
            list.appendChild(item);
        });
        function show(open) {
            list.style.display = open ? "block" : "none";
            button.setAttribute("aria-expanded", open ? "true" : "false");
        }
        button.addEventListener("click", function (e) {
            e.stopPropagation();
            show(list.style.display !== "block");
        });
        document.addEventListener("click", function (e) {
            if (!wrap.contains(e.target)) {
                show(false);
            }
        });
        document.addEventListener("keydown", function (e) {
            if (e.key === "Escape" && list.style.display === "block") {
                show(false);
                button.focus();
            }
        });
        wrap.appendChild(button);
        wrap.appendChild(list);
        bar.insertBefore(wrap, bar.firstChild);
    }

    function notice() {
        var main = document.querySelector("#mdbook-content main") || document.querySelector("main");
        // The start page has the notice in its source, which works without
        // JavaScript; do not repeat it there.
        if (!data.notice || !main) {
            return;
        }
        var english = data.languages[0];
        var present = main.querySelector(".ai-notice");
        if (present) {
            // Its link to English counts as a choice too.
            Array.prototype.forEach.call(present.querySelectorAll("a"), function (link) {
                anchored.push({ link: link, lang: english });
                chosen(link, english);
            });
            refresh();
            return;
        }
        var box = document.createElement("div");
        box.className = "ai-notice";
        box.setAttribute("role", "note");
        box.appendChild(document.createTextNode(data.notice.text + " "));
        var link = document.createElement("a");
        link.href = href(english);
        link.hreflang = "en";
        anchored.push({ link: link, lang: english });
        chosen(link, english);
        link.textContent = data.notice.link;
        box.appendChild(link);
        main.insertBefore(box, main.firstChild);
    }

    detect();
    languageMenu();
    notice();
    window.addEventListener("hashchange", refresh);
})();
