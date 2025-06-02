import { Widget } from '@seelen-ui/lib';

const widget = await Widget.getCurrentAsync();

const { js, css, html } = widget.def;

if (html) {
  document.body.innerHTML = html;
}

if (css) {
  const style = document.createElement('style');
  style.textContent = css;
  document.head.appendChild(style);
}

if (js) {
  const script = document.createElement('script');
  script.type = 'module';
  script.textContent = js;
  document.head.appendChild(script);
}
