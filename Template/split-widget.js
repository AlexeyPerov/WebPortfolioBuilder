(function () {
  function readSiteWidgetsConfig() {
    var el = document.getElementById("site-widgets-config");
    if (!el) {
      return {};
    }
    try {
      return JSON.parse(el.textContent || "{}");
    } catch (err) {
      return {};
    }
  }

  function activateTab(widget, tab) {
    var id = tab.getAttribute("data-target");
    if (!id) return;
    var tabs = widget.querySelectorAll(".split-widget__tab");
    var panels = widget.querySelectorAll(".split-widget__panel");
    tabs.forEach(function (t) {
      t.classList.remove("is-active");
      t.setAttribute("aria-selected", "false");
    });
    panels.forEach(function (p) {
      p.classList.remove("is-active");
    });
    tab.classList.add("is-active");
    tab.setAttribute("aria-selected", "true");
    var panel = widget.querySelector("#" + id);
    if (panel) {
      panel.classList.add("is-active");
    }
  }

  function tabIndexActive(tabs) {
    for (var i = 0; i < tabs.length; i++) {
      if (tabs[i].classList.contains("is-active")) return i;
    }
    return 0;
  }

  function initWidget(widget) {
    var tabs = widget.querySelectorAll(".split-widget__tab");
    tabs.forEach(function (tab) {
      tab.addEventListener("click", function () {
        activateTab(widget, tab);
      });
    });

    var cfg = readSiteWidgetsConfig();
    var keyboard =
      cfg.split_widget &&
      cfg.split_widget.keyboard_navigation === true;
    if (!keyboard || !tabs.length) {
      return;
    }

    widget.setAttribute("tabindex", "0");

    widget.addEventListener("keydown", function (e) {
      if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") {
        return;
      }
      var list = widget.querySelectorAll(".split-widget__tab");
      if (!list.length) return;
      var tabsArr = Array.prototype.slice.call(list);
      var i = tabIndexActive(tabsArr);
      var next =
        e.key === "ArrowRight"
          ? i + 1
          : i - 1;
      next = ((next % tabsArr.length) + tabsArr.length) % tabsArr.length;
      activateTab(widget, tabsArr[next]);
      tabsArr[next].focus();
      e.preventDefault();
    });
  }

  document.querySelectorAll("[data-split-widget]").forEach(initWidget);
})();
