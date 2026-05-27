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

  var defaults = {
    respect_reduced_motion: true,
    root_margin: "0px 0px -5% 0px",
    threshold: 0.12,
  };

  var raw = readSiteWidgetsConfig();
  var sr = raw.scroll_reveal || {};
  var respectReduced =
    sr.respect_reduced_motion !== undefined
      ? !!sr.respect_reduced_motion
      : defaults.respect_reduced_motion;
  var rootMargin =
    typeof sr.root_margin === "string" && sr.root_margin.trim() !== ""
      ? sr.root_margin
      : defaults.root_margin;
  var threshold =
    typeof sr.threshold === "number" ? sr.threshold : defaults.threshold;

  if (
    respectReduced &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches
  ) {
    return;
  }

  var nodes = document.querySelectorAll(".scroll-reveal:not(.scroll-reveal--immediate)");
  var immediate = document.querySelectorAll(".scroll-reveal--immediate");
  for (var k = 0; k < immediate.length; k++) {
    immediate[k].classList.add("scroll-reveal--visible");
  }
  if (!nodes.length) {
    return;
  }

  var observer = new IntersectionObserver(
    function (entries) {
      for (var i = 0; i < entries.length; i++) {
        var e = entries[i];
        if (e.isIntersecting) {
          e.target.classList.add("scroll-reveal--visible");
          observer.unobserve(e.target);
        }
      }
    },
    { root: null, rootMargin: rootMargin, threshold: threshold }
  );

  for (var j = 0; j < nodes.length; j++) {
    observer.observe(nodes[j]);
  }
})();
