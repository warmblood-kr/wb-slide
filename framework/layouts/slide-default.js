import { SlideBase } from '../slide-base.js';

class SlideDefault extends SlideBase {
  layoutTemplate(content) {
    return `<div class="ms-default-layout">${content}</div>`;
  }
}

customElements.define('slide-default', SlideDefault);
