(function () {
  function activatePanel(widget, targetId) {
    if (!targetId) return;
    var tabs = widget.querySelectorAll(".split-widget__tab");
    var panels = widget.querySelectorAll(".split-widget__panel");
    tabs.forEach(function (t) {
      var match = t.getAttribute("data-target") === targetId;
      t.classList.toggle("is-active", match);
      t.setAttribute("aria-selected", match ? "true" : "false");
    });
    panels.forEach(function (p) {
      p.classList.toggle("is-active", p.id === targetId);
    });
    var select = widget.querySelector(".reference-panel__select");
    if (select && select.value !== targetId) {
      select.value = targetId;
    }
  }

  function initWidget(widget) {
    var select = widget.querySelector(".reference-panel__select");
    var tabs = widget.querySelectorAll(".split-widget__tab");

    tabs.forEach(function (tab) {
      tab.addEventListener("click", function () {
        activatePanel(widget, tab.getAttribute("data-target"));
      });
    });

    if (select) {
      select.addEventListener("change", function () {
        activatePanel(widget, select.value);
      });
    }
  }

  document.querySelectorAll("[data-reference-panel]").forEach(initWidget);
})();
