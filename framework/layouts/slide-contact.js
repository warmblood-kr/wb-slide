import { SlideBase } from '../slide-base.js';

class SlideContact extends SlideBase {
  layoutTemplate(content) {
    return `<div class="ms-contact-layout">${content}</div>`;
  }
}

customElements.define('slide-contact', SlideContact);
