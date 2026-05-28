import { SlideBase } from '../slide-base.js';

class SlideTwoColumn extends SlideBase {
  layoutTemplate(content, slots) {
    const heading = this.getAttribute('heading') || '';
    const subtitle = this.getAttribute('subtitle') || '';
    const left = slots.left || '';
    const right = slots.right || '';

    return `
      <div class="ms-two-column-outer">
        ${heading ? `<h1 class="ms-slide-title">${heading}</h1>` : ''}
        ${subtitle ? `<p class="ms-slide-subtitle">${subtitle}</p>` : ''}
        <div class="ms-two-column-layout">
          <div class="ms-col">${left || content}</div>
          <div class="ms-col">${right}</div>
        </div>
      </div>
    `;
  }
}

customElements.define('slide-two-column', SlideTwoColumn);
