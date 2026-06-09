// wb-slide: client-side navigation & scaling for SSR-rendered slides.
// All slide layout HTML is rendered server-side by Rust.
// This script only handles: viewport scaling, keyboard navigation, and hash routing.
(function () {
  'use strict';
  var deck = document.getElementById('monocle-slide-deck');
  if (!deck) return;
  var viewport = deck.querySelector('.ms-viewport');
  if (!viewport) return;
  var slides = Array.prototype.slice.call(deck.querySelectorAll('.ms-slide-container'));
  if (slides.length === 0) return;

  var current = 0;

  function show(i) {
    if (i < 0) i = 0;
    if (i >= slides.length) i = slides.length - 1;
    for (var k = 0; k < slides.length; k++) {
      if (k === i) slides[k].classList.add('active');
      else slides[k].classList.remove('active');
    }
    current = i;
    var hash = '#/' + (i + 1);
    if (window.location.hash !== hash) {
      history.replaceState(null, '', hash);
    }
  }

  function slideFromHash() {
    var m = window.location.hash.match(/^#\/(\d+)/);
    if (m) return parseInt(m[1], 10) - 1;
    return 0;
  }

  function rescale() {
    var sx = window.innerWidth / 960;
    var sy = window.innerHeight / 540;
    var s = Math.min(sx, sy);
    viewport.style.transform = 'translate(-50%, -50%) scale(' + s + ')';
  }

  window.addEventListener('resize', rescale);
  window.addEventListener('hashchange', function () {
    var t = slideFromHash();
    if (t !== current) show(t);
  });

  window.addEventListener('keydown', function (e) {
    switch (e.key) {
      case 'ArrowRight':
      case 'ArrowDown':
      case ' ':
      case 'PageDown':
        e.preventDefault();
        show(current + 1);
        break;
      case 'ArrowLeft':
      case 'ArrowUp':
      case 'PageUp':
        e.preventDefault();
        show(current - 1);
        break;
      case 'Home':
        e.preventDefault();
        show(0);
        break;
      case 'End':
        e.preventDefault();
        show(slides.length - 1);
        break;
      case 'f':
      case 'F':
        if (!e.ctrlKey && !e.metaKey) {
          e.preventDefault();
          if (document.fullscreenElement) document.exitFullscreen();
          else document.documentElement.requestFullscreen();
        }
        break;
    }
  });

  rescale();
  show(slideFromHash());
})();
