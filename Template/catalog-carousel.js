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

  function swipeThresholdPx(cfg) {
    var def = 30;
    var v =
      cfg.carousel &&
      typeof cfg.carousel.swipe_threshold_px === "number"
        ? cfg.carousel.swipe_threshold_px
        : def;
    return v > 0 ? v : def;
  }

  function keyboardNavigationEnabled(cfg) {
    if (!cfg.carousel || cfg.carousel.keyboard_navigation === undefined) {
      return true;
    }
    return !!cfg.carousel.keyboard_navigation;
  }

  function initSwiper(root) {
    var slides = Array.prototype.slice.call(
      root.querySelectorAll(".catalog-carousel__slide")
    );
    if (slides.length === 0) {
      return;
    }
    var cfg = readSiteWidgetsConfig();
    var prevBtn = root.querySelector(".catalog-carousel__arrow--prev");
    var nextBtn = root.querySelector(".catalog-carousel__arrow--next");
    var active = 0;
    var threshold = swipeThresholdPx(cfg);
    var status = null;
    var live = null;

    if (slides.length > 1) {
      status = document.createElement("p");
      status.className = "catalog-carousel__status";
      status.setAttribute("aria-hidden", "true");
      root.appendChild(status);

      live = document.createElement("div");
      live.className = "catalog-carousel__live";
      live.setAttribute("aria-live", "polite");
      live.setAttribute("aria-atomic", "true");
      root.appendChild(live);
    }

    function mod(n, m) {
      return ((n % m) + m) % m;
    }

    function slideAlt(index) {
      var img = slides[index].querySelector("img");
      if (!img) {
        return "";
      }
      var alt = img.getAttribute("alt");
      return alt && alt.trim() ? alt.trim() : "";
    }

    function updateStatus() {
      if (!status || !live || slides.length <= 1) {
        return;
      }
      var current = active + 1;
      var total = slides.length;
      status.textContent = current + " / " + total;
      var msg = "Slide " + current + " of " + total;
      var alt = slideAlt(active);
      if (alt) {
        msg += ": " + alt;
      }
      live.textContent = msg;
    }

    function render() {
      for (var i = 0; i < slides.length; i++) {
        slides[i].classList.remove("state-prev", "state-active", "state-next");
      }
      slides[active].classList.add("state-active");
      if (slides.length > 1) {
        slides[mod(active - 1, slides.length)].classList.add("state-prev");
        slides[mod(active + 1, slides.length)].classList.add("state-next");
      }
      updateStatus();
    }

    function go(delta) {
      if (slides.length <= 1) {
        return;
      }
      active = mod(active + delta, slides.length);
      render();
    }

    if (prevBtn) {
      prevBtn.addEventListener("click", function () {
        go(-1);
      });
    }
    if (nextBtn) {
      nextBtn.addEventListener("click", function () {
        go(1);
      });
    }

    if (keyboardNavigationEnabled(cfg) && slides.length > 1) {
      root.setAttribute("tabindex", "0");
      root.addEventListener("keydown", function (e) {
        if (e.key === "ArrowLeft") {
          go(-1);
          e.preventDefault();
        } else if (e.key === "ArrowRight") {
          go(1);
          e.preventDefault();
        }
      });
    }

    var startX = null;
    var viewport = root.querySelector(".catalog-carousel__viewport");
    if (viewport) {
      viewport.addEventListener(
        "touchstart",
        function (e) {
          if (e.touches && e.touches.length) {
            startX = e.touches[0].clientX;
          }
        },
        { passive: true }
      );
      viewport.addEventListener(
        "touchend",
        function (e) {
          if (
            startX === null ||
            !e.changedTouches ||
            !e.changedTouches.length
          ) {
            return;
          }
          var endX = e.changedTouches[0].clientX;
          var delta = endX - startX;
          startX = null;
          if (Math.abs(delta) < threshold) {
            return;
          }
          go(delta < 0 ? 1 : -1);
        },
        { passive: true }
      );
    }

    render();
  }

  var swipers = document.querySelectorAll("[data-catalog-carousel]");
  for (var i = 0; i < swipers.length; i++) {
    initSwiper(swipers[i]);
  }
})();
