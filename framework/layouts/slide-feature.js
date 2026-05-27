import { SlideBase } from '../slide-base.js';

class SlideFeature extends SlideBase {
  layoutTemplate(content) {
    const heading = this.getAttribute('heading') || '';
    const subtitle = this.getAttribute('subtitle') || '';
    return `
      <div class="ms-feature-layout">
        <h1 class="ms-slide-title">${heading}</h1>
        ${subtitle ? `<p class="ms-slide-subtitle">${subtitle}</p>` : ''}
        <div class="ms-slot-area">${content}</div>
      </div>
    `;
  }
}

customElements.define('slide-feature', SlideFeature);
