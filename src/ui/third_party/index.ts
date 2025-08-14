import type { Widget } from '@seelen-ui/lib/types';
import { removeDefaultWebviewActions } from '@shared/setup';
import { _invoke, WebviewInformation } from 'libs/widgets-integrity/_tauri';

const currentWidgetId = new WebviewInformation().widgetId;
const widgetList = await _invoke<Widget[]>('state_get_widgets');
const widget = widgetList.find((widget) => widget.id === currentWidgetId)!;

removeDefaultWebviewActions();

const { js, css, html } = widget;

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
