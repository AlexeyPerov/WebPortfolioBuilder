(function () {
  var toggle = document.querySelector(".site-nav-toggle");
  var nav = document.getElementById("site-nav");

  if (toggle && nav) {
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
  }

  if (!nav) {
    return;
  }

  var hashLinks = nav.querySelectorAll('.site-nav__link[href^="#"]');
  if (hashLinks.length === 0) {
    return;
  }

  var sections = [];
  var sectionById = {};

  hashLinks.forEach(function (link) {
    var hash = link.getAttribute("href");
    if (!hash || hash.length < 2) {
      return;
    }
    var id = decodeURIComponent(hash.slice(1));
    if (sectionById[id]) {
      return;
    }
    var target = document.getElementById(id);
    if (!target) {
      return;
    }
    sectionById[id] = target;
    sections.push(target);
  });

  if (sections.length === 0) {
    return;
  }

  var activeId = null;
  var intersecting = [];

  function setActiveSection(id) {
    if (activeId === id) {
      return;
    }
    activeId = id;
    hashLinks.forEach(function (link) {
      var hash = link.getAttribute("href") || "";
      var linkId = hash.length > 1 ? decodeURIComponent(hash.slice(1)) : "";
      link.classList.toggle("site-nav__link--active", linkId === id);
      if (linkId === id) {
        link.setAttribute("aria-current", "location");
      } else {
        link.removeAttribute("aria-current");
      }
    });
  }

  function pickActiveSection() {
    if (intersecting.length === 0) {
      return;
    }
    intersecting.sort(function (a, b) {
      return sections.indexOf(a) - sections.indexOf(b);
    });
    setActiveSection(intersecting[intersecting.length - 1].id);
  }

  if ("IntersectionObserver" in window) {
    var observer = new IntersectionObserver(
      function (entries) {
        entries.forEach(function (entry) {
          var idx = intersecting.indexOf(entry.target);
          if (entry.isIntersecting) {
            if (idx === -1) {
              intersecting.push(entry.target);
            }
          } else if (idx !== -1) {
            intersecting.splice(idx, 1);
          }
        });
        pickActiveSection();
      },
      {
        rootMargin: "-84px 0px -55% 0px",
        threshold: 0,
      }
    );

    sections.forEach(function (section) {
      observer.observe(section);
    });
  }

  hashLinks.forEach(function (link) {
    link.addEventListener("click", function () {
      var hash = link.getAttribute("href") || "";
      if (hash.length < 2) {
        return;
      }
      var id = decodeURIComponent(hash.slice(1));
      if (sectionById[id]) {
        setActiveSection(id);
      }
    });
  });
})();
