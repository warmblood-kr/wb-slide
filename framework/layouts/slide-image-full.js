import { SlideBase } from '../slide-base.js';

class SlideImageFull extends SlideBase {
  showChrome() {
    return false;
  }

  layoutTemplate(content) {
    return `<div class="ms-image-full-layout">${content}</div>`;
  }
}

customElements.define('slide-image-full', SlideImageFull);
