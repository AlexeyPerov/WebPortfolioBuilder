(function () {
  function activateTab(widget, tab) {
    var id = tab.getAttribute('data-target');
    if (!id) return;
    var tabs = widget.querySelectorAll('.split-widget__tab');
    var panels = widget.querySelectorAll('.split-widget__panel');
    tabs.forEach(function (t) {
      t.classList.remove('is-active');
      t.setAttribute('aria-selected', 'false');
    });
    panels.forEach(function (p) {
      p.classList.remove('is-active');
    });
    tab.classList.add('is-active');
    tab.setAttribute('aria-selected', 'true');
    var panel = widget.querySelector('#' + id);
    if (panel) {
      panel.classList.add('is-active');
    }
  }

  function initWidget(widget) {
    var tabs = widget.querySelectorAll('.split-widget__tab');
    tabs.forEach(function (tab) {
      tab.addEventListener('click', function () {
        activateTab(widget, tab);
      });
    });
  }

  document.querySelectorAll('[data-split-widget]').forEach(initWidget);
})();
