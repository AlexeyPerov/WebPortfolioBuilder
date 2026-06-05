(function () {
  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    return;
  }

  var root = document.querySelector(".page-bg-effect--magic-dust");
  if (!root) {
    return;
  }

  var canvas = root.querySelector(".page-bg-effect__canvas");
  if (!canvas) {
    return;
  }

  var ctx = canvas.getContext("2d");
  if (!ctx) {
    return;
  }

  var particles = [];
  var particleCount = 120;
  var running = true;
  var dpr = Math.min(window.devicePixelRatio || 1, 2);

  function resize() {
    var w = window.innerWidth;
    var h = window.innerHeight;
    canvas.width = Math.floor(w * dpr);
    canvas.height = Math.floor(h * dpr);
    canvas.style.width = w + "px";
    canvas.style.height = h + "px";
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  }

  function randomBetween(min, max) {
    return min + Math.random() * (max - min);
  }

  function initParticles() {
    particles = [];
    var w = window.innerWidth;
    var h = window.innerHeight;
    for (var i = 0; i < particleCount; i += 1) {
      particles.push({
        x: Math.random() * w,
        y: Math.random() * h,
        radius: randomBetween(0.6, 1.8),
        speedY: randomBetween(-0.25, -0.08),
        speedX: randomBetween(-0.06, 0.06),
        phase: Math.random() * Math.PI * 2,
        twinkle: randomBetween(0.015, 0.04),
        accent: Math.random() < 0.18,
      });
    }
  }

  function draw() {
    if (!running) {
      return;
    }

    var w = window.innerWidth;
    var h = window.innerHeight;
    ctx.clearRect(0, 0, w, h);

    for (var i = 0; i < particles.length; i += 1) {
      var p = particles[i];
      p.x += p.speedX;
      p.y += p.speedY;
      p.phase += p.twinkle;

      if (p.y < -4) {
        p.y = h + 4;
        p.x = Math.random() * w;
      }
      if (p.x < -4) {
        p.x = w + 4;
      } else if (p.x > w + 4) {
        p.x = -4;
      }

      var alpha = 0.32 + Math.sin(p.phase) * 0.22;
      if (p.accent) {
        ctx.fillStyle = "rgba(77, 141, 255, " + alpha.toFixed(3) + ")";
      } else {
        ctx.fillStyle = "rgba(244, 246, 251, " + (alpha * 0.9).toFixed(3) + ")";
      }

      ctx.beginPath();
      ctx.arc(p.x, p.y, p.radius, 0, Math.PI * 2);
      ctx.fill();
    }

    window.requestAnimationFrame(draw);
  }

  resize();
  initParticles();
  window.requestAnimationFrame(draw);

  window.addEventListener("resize", function () {
    resize();
    initParticles();
  });

  document.addEventListener("visibilitychange", function () {
    running = !document.hidden;
    if (running) {
      window.requestAnimationFrame(draw);
    }
  });
})();
