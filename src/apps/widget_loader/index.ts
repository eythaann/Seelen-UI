import { WidgetList } from '@seelen-ui/lib';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

import { wrapConsole } from '../shared/ConsoleWrapper';

wrapConsole();

const webview = getCurrentWebviewWindow();
const base64Label = webview.label;
const decodedLabel = atob(base64Label);

const list = await WidgetList.getAsync();
const widget = list.all().find((w) => w.id === decodedLabel);

if (widget) {
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
}
