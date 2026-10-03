// Tabs for the download section, opened on the visitor's operating system.
// Without JavaScript, or on an unknown system, every section stays visible.
(function () {
  var container = document.getElementById("tabs");
  if (!container) return;
  var sections = Array.prototype.slice.call(container.querySelectorAll(":scope > section[data-os]"));
  if (sections.length === 0) return;

  function detect() {
    var ua = navigator.userAgent || "";
    if (/Android/i.test(ua)) return null;
    var platform = (navigator.userAgentData && navigator.userAgentData.platform) || "";
    var text = platform || ua;
    if (/Android/i.test(platform)) return null;
    if (/Win/i.test(text)) return "windows";
    if (/Mac/i.test(text)) return "macos";
    if (/Linux|X11/i.test(text)) return "linux";
    return null;
  }

  var current = detect();
  if (!current) return;

  var list = document.createElement("div");
  list.setAttribute("role", "tablist");
  list.setAttribute("aria-label", "Operating system");
  list.className = "tablist";
  var tabs = [];

  function select(os, focus) {
    tabs.forEach(function (tab) {
      var on = tab.dataset.os === os;
      tab.setAttribute("aria-selected", on ? "true" : "false");
      tab.tabIndex = on ? 0 : -1;
      if (on && focus) tab.focus();
    });
    sections.forEach(function (s) { s.hidden = s.dataset.os !== os; });
  }

  sections.forEach(function (section) {
    var heading = section.querySelector("h3");
    var tab = document.createElement("button");
    tab.type = "button";
    tab.setAttribute("role", "tab");
    tab.id = "tab-" + section.dataset.os;
    tab.dataset.os = section.dataset.os;
    tab.setAttribute("aria-controls", section.id);
    tab.textContent = heading ? heading.textContent : section.dataset.os;
    tab.addEventListener("click", function () { select(section.dataset.os, false); });
    tab.addEventListener("keydown", function (e) {
      var i = tabs.indexOf(tab), n = tabs.length, next = -1;
      if (e.key === "ArrowRight") next = (i + 1) % n;
      else if (e.key === "ArrowLeft") next = (i + n - 1) % n;
      else if (e.key === "Home") next = 0;
      else if (e.key === "End") next = n - 1;
      if (next >= 0) { e.preventDefault(); select(tabs[next].dataset.os, true); }
    });
    list.appendChild(tab);
    tabs.push(tab);
    section.setAttribute("role", "tabpanel");
    section.setAttribute("aria-labelledby", tab.id);
    if (heading) heading.hidden = true;
  });

  container.insertBefore(list, container.firstChild);
  select(current, false);
})();
