export class SlideBase extends HTMLElement {
  connectedCallback() {
    const index = this.getAttribute('slide-index') || '';
    const content = this.innerHTML;
    const slots = this._slots || {};

    const watermark = this.getAttribute('watermark') || '';
    const footer = this.getAttribute('footer') || '';

    const chrome = this.showChrome() ? `
      ${watermark ? `<div class="ms-watermark">${watermark}</div>` : ''}
      ${footer ? `<div class="ms-footer-logo">${footer}</div>` : ''}
      <div class="ms-page-number">${index}</div>
    ` : '';

    this.innerHTML = chrome + this.layoutTemplate(content, slots);
  }

  layoutTemplate(content, slots) {
    return `<div class="ms-default-layout">${content}</div>`;
  }

  showChrome() {
    return true;
  }
}
