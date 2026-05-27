import './layouts/slide-default.js';
import './layouts/slide-feature.js';
import './layouts/slide-cover.js';
import './layouts/slide-section.js';
import './layouts/slide-contact.js';
import './layouts/slide-two-column.js';
import './layouts/slide-image-full.js';
import './layouts/slide-quote.js';

class MonocleSlide extends HTMLElement {
  #currentSlide = 0;
  #containers = [];

  connectedCallback() {
    this.innerHTML = `<div class="ms-viewport"></div>`;
    const viewport = this.querySelector('.ms-viewport');

    const slides = window.__MONOCLE_SLIDES__ || [];

    slides.forEach((slide) => {
      const container = document.createElement('div');
      container.className = 'ms-slide-container';

      const el = document.createElement(slide.layout || 'slide-default');
      el.setAttribute('slide-index', String(slide.index));
      el.setAttribute('total-slides', String(slides.length));

      for (const [key, value] of Object.entries(slide.attrs || {})) {
        el.setAttribute(key, value);
      }

      el.innerHTML = slide.body || '';

      container.appendChild(el);
      viewport.appendChild(container);
      this.#containers.push(container);
    });

    this.#setupScale();
    this.#setupKeyboard();
    this.#goTo(this.#slideFromHash());
  }

  #goTo(index) {
    if (index < 0) index = 0;
    if (index >= this.#containers.length) index = this.#containers.length - 1;

    this.#containers.forEach((c, i) => {
      c.classList.toggle('active', i === index);
    });

    this.#currentSlide = index;
    history.replaceState(null, '', `#/${index + 1}`);
  }

  #slideFromHash() {
    const match = window.location.hash.match(/#\/(\d+)/);
    if (match) return parseInt(match[1], 10) - 1;
    return 0;
  }

  #setupScale() {
    const viewport = this.querySelector('.ms-viewport');
    const rescale = () => {
      const scaleX = window.innerWidth / 960;
      const scaleY = window.innerHeight / 540;
      const scale = Math.min(scaleX, scaleY);
      viewport.style.transform = `translate(-50%, -50%) scale(${scale})`;
    };
    rescale();
    window.addEventListener('resize', rescale);
  }

  #setupKeyboard() {
    window.addEventListener('keydown', (e) => {
      switch (e.key) {
        case 'ArrowRight': case 'ArrowDown': case ' ': case 'PageDown':
          e.preventDefault();
          this.#goTo(this.#currentSlide + 1);
          break;
        case 'ArrowLeft': case 'ArrowUp': case 'PageUp':
          e.preventDefault();
          this.#goTo(this.#currentSlide - 1);
          break;
        case 'Home':
          e.preventDefault();
          this.#goTo(0);
          break;
        case 'End':
          e.preventDefault();
          this.#goTo(this.#containers.length - 1);
          break;
        case 'f': case 'F':
          if (!e.ctrlKey && !e.metaKey) {
            e.preventDefault();
            document.fullscreenElement
              ? document.exitFullscreen()
              : document.documentElement.requestFullscreen();
          }
          break;
      }
    });

    window.addEventListener('hashchange', () => {
      const target = this.#slideFromHash();
      if (target !== this.#currentSlide) this.#goTo(target);
    });
  }
}

customElements.define('monocle-slide', MonocleSlide);
