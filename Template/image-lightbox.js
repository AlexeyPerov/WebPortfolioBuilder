(function () {
  var grids = document.querySelectorAll("[data-image-lightbox]");
  if (!grids.length) {
    return;
  }

  var overlay = document.createElement("div");
  overlay.className = "image-lightbox";
  overlay.hidden = true;
  overlay.innerHTML =
    '<div class="image-lightbox__backdrop" data-lightbox-close tabindex="-1"></div>' +
    '<div class="image-lightbox__dialog" role="dialog" aria-modal="true" aria-label="Enlarged image">' +
    '<button type="button" class="image-lightbox__close" data-lightbox-close aria-label="Close image">×</button>' +
    '<img class="image-lightbox__image" alt="">' +
    "</div>";
  document.body.appendChild(overlay);

  var dialog = overlay.querySelector(".image-lightbox__dialog");
  var image = overlay.querySelector(".image-lightbox__image");
  var closeBtn = overlay.querySelector(".image-lightbox__close");
  var lastFocus = null;

  function focusables() {
    return overlay.querySelectorAll("[data-lightbox-close], .image-lightbox__image");
  }

  function trapFocus(e) {
    if (e.key !== "Tab" || overlay.hidden) {
      return;
    }
    var items = focusables();
    if (!items.length) {
      return;
    }
    var first = items[0];
    var last = items[items.length - 1];
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  }

  function closeLightbox() {
    if (overlay.hidden) {
      return;
    }
    overlay.hidden = true;
    document.body.classList.remove("image-lightbox-open");
    image.removeAttribute("src");
    image.alt = "";
    if (lastFocus && typeof lastFocus.focus === "function") {
      lastFocus.focus();
    }
  }

  function openLightbox(src, alt, trigger) {
    lastFocus = trigger;
    image.src = src;
    image.alt = alt || "";
    overlay.hidden = false;
    document.body.classList.add("image-lightbox-open");
    closeBtn.focus();
  }

  document.addEventListener("keydown", function (e) {
    if (e.key === "Escape") {
      closeLightbox();
      return;
    }
    trapFocus(e);
  });

  overlay.addEventListener("click", function (e) {
    if (e.target.matches("[data-lightbox-close]")) {
      closeLightbox();
    }
  });

  grids.forEach(function (grid) {
    grid.addEventListener("click", function (e) {
      var item = e.target.closest(".photos-grid__item");
      if (!item) {
        return;
      }
      var src = item.getAttribute("data-lightbox-src");
      if (!src) {
        return;
      }
      openLightbox(src, item.getAttribute("data-lightbox-alt") || "", item);
    });
  });
})();
