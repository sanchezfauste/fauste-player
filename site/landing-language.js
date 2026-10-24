// The language of the landing page and the privacy policy. The menu is
// plain links rendered by scripts/site/landing.py, so it works without
// JavaScript; this script stores a choice made in it (or by the AI notice's
// link to English) and, on an English page, opens the visitor's language.
// The decision and the stored choice are shared with the user guide
// (site-language.js).
(function () {
  "use strict";
  var shared = window.faustePlayerLanguage;
  if (!shared) return;

  var targets = {};
  var links = document.querySelectorAll(".language-menu a[hreflang], .ai-notice a[hreflang]");
  Array.prototype.forEach.call(links, function (link) {
    var code = link.getAttribute("hreflang");
    if (link.closest(".language-menu")) targets[code] = link.href;
    link.addEventListener("click", function () { shared.remember(code); });
  });

  var choice = shared.decide({
    current: document.documentElement.lang,
    codes: Object.keys(targets).filter(function (code) { return code !== "en"; }),
    search: window.location.search,
    stored: shared.stored(),
    wanted: navigator.languages || (navigator.language ? [navigator.language] : [])
  });
  if (choice.remember) shared.remember(choice.remember);
  if (choice.go && targets[choice.go]) {
    window.location.replace(targets[choice.go] + window.location.hash);
  }
})();
