// The language menu in the guide's top bar and, on a translated page, the
// notice that it was translated with AI. The data (languages, the pages each
// one has, the notice) comes from languages.js, which scripts/site/guide.py
// generates for every book. Uses no external resources.
(function () {
    "use strict";
    var data = window.faustePlayerGuide;
    if (!data || typeof path_to_root === "undefined") {
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

    function href(lang) {
        var base = lang.code === "en" ? guideRoot : new URL(lang.code + "/", guideRoot);
        // A page the other language does not have yet opens its start page.
        // Translations keep the English heading ids, so the anchor carries over.
        if (lang.pages.indexOf(page) < 0) {
            return new URL("index.html", base).href;
        }
        return new URL(page + window.location.hash, base).href;
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
            link.lang = lang.code;
            link.hreflang = lang.code;
            link.textContent = lang.name;
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
        if (!data.notice || !main || main.querySelector(".ai-notice")) {
            return;
        }
        var english = data.languages[0];
        var box = document.createElement("div");
        box.className = "ai-notice";
        box.setAttribute("role", "note");
        box.appendChild(document.createTextNode(data.notice.text + " "));
        var link = document.createElement("a");
        link.href = href(english);
        link.hreflang = "en";
        link.textContent = data.notice.link;
        box.appendChild(link);
        main.insertBefore(box, main.firstChild);
    }

    languageMenu();
    notice();
})();
