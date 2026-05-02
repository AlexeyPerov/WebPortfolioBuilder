(function () {
  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    return;
  }
  var nodes = document.querySelectorAll(".scroll-reveal");
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
    { root: null, rootMargin: "0px 0px -5% 0px", threshold: 0.12 }
  );
  for (var j = 0; j < nodes.length; j++) {
    observer.observe(nodes[j]);
  }
})();
