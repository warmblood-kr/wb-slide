import { SlideBase } from '../slide-base.js';

class SlideTwoColumn extends SlideBase {
  layoutTemplate(content) {
    return `<div class="ms-two-column-layout">${content}</div>`;
  }
}

customElements.define('slide-two-column', SlideTwoColumn);
