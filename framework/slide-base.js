export class SlideBase extends HTMLElement {
  connectedCallback() {
    const index = this.getAttribute('slide-index') || '';
    const content = this.innerHTML;

    const chrome = this.showChrome() ? `
      <div class="ms-watermark">${this.watermarkText()}</div>
      <div class="ms-footer-logo"><i>W</i><span>armblood</span></div>
      <div class="ms-page-number">${index}</div>
    ` : '';

    this.innerHTML = chrome + this.layoutTemplate(content);
  }

  layoutTemplate(content) {
    return `<div class="ms-default-layout">${content}</div>`;
  }

  showChrome() {
    return true;
  }

  watermarkText() {
    return 'Monocle AI';
  }
}
