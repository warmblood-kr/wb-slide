import { SlideBase } from '../slide-base.js';

class SlideSection extends SlideBase {
  layoutTemplate(content) {
    return `<div class="ms-section-layout">${content}</div>`;
  }
}

customElements.define('slide-section', SlideSection);
