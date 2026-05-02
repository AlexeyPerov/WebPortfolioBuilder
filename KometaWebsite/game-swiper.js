(function () {
  function initSwiper(root) {
    var slides = Array.prototype.slice.call(root.querySelectorAll(".game-swiper__slide"));
    if (slides.length === 0) {
      return;
    }
    var prevBtn = root.querySelector(".game-swiper__arrow--prev");
    var nextBtn = root.querySelector(".game-swiper__arrow--next");
    var active = 0;

    function mod(n, m) {
      return ((n % m) + m) % m;
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
    }

    function go(delta) {
      if (slides.length <= 1) {
        return;
      }
      active = mod(active + delta, slides.length);
      render();
    }

    if (prevBtn) {
      prevBtn.addEventListener("click", function () { go(-1); });
    }
    if (nextBtn) {
      nextBtn.addEventListener("click", function () { go(1); });
    }

    var startX = null;
    var viewport = root.querySelector(".game-swiper__viewport");
    if (viewport) {
      viewport.addEventListener("touchstart", function (e) {
        if (e.touches && e.touches.length) {
          startX = e.touches[0].clientX;
        }
      }, { passive: true });
      viewport.addEventListener("touchend", function (e) {
        if (startX === null || !e.changedTouches || !e.changedTouches.length) {
          return;
        }
        var endX = e.changedTouches[0].clientX;
        var delta = endX - startX;
        startX = null;
        if (Math.abs(delta) < 30) {
          return;
        }
        go(delta < 0 ? 1 : -1);
      }, { passive: true });
    }

    render();
  }

  var swipers = document.querySelectorAll("[data-game-swiper]");
  for (var i = 0; i < swipers.length; i++) {
    initSwiper(swipers[i]);
  }
})();
