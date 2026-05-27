(function () {
  var toggle = document.querySelector(".site-nav-toggle");
  var nav = document.getElementById("site-nav");
  if (!toggle || !nav) {
    return;
  }

  // Focus trap omitted: mobile nav is a short in-page link list; Escape closes and returns focus to toggle.
  var mq = window.matchMedia("(max-width: 720px)");

  function setOpen(open) {
    nav.classList.toggle("site-nav--open", open);
    toggle.setAttribute("aria-expanded", open ? "true" : "false");
    toggle.setAttribute("aria-label", open ? "Close menu" : "Menu");
  }

  function close() {
    setOpen(false);
  }

  toggle.addEventListener("click", function () {
    setOpen(!nav.classList.contains("site-nav--open"));
  });

  document.addEventListener("keydown", function (e) {
    if (e.key === "Escape" && nav.classList.contains("site-nav--open")) {
      close();
      toggle.focus();
    }
  });

  nav.addEventListener("click", function (e) {
    var link = e.target.closest(".site-nav__link");
    if (link && mq.matches) {
      close();
    }
  });

  if (typeof mq.addEventListener === "function") {
    mq.addEventListener("change", function () {
      if (!mq.matches) {
        close();
      }
    });
  } else if (typeof mq.addListener === "function") {
    mq.addListener(function () {
      if (!mq.matches) {
        close();
      }
    });
  }
})();
