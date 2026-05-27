import { SlideBase } from '../slide-base.js';

class SlideCover extends SlideBase {
  showChrome() {
    return false;
  }

  layoutTemplate(content) {
    return `<div class="ms-cover-layout">${content}</div>`;
  }
}

customElements.define('slide-cover', SlideCover);
