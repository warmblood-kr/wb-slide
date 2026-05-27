import { SlideBase } from '../slide-base.js';

class SlideQuote extends SlideBase {
  layoutTemplate(content) {
    const quote = this.getAttribute('quote') || '';
    const author = this.getAttribute('author') || '';
    if (quote) {
      return `
        <div class="ms-quote-layout">
          <blockquote class="ms-quote-text">${quote}</blockquote>
          ${author ? `<cite class="ms-quote-author">— ${author}</cite>` : ''}
          ${content}
        </div>
      `;
    }
    return `<div class="ms-quote-layout">${content}</div>`;
  }
}

customElements.define('slide-quote', SlideQuote);
