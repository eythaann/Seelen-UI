import { disableAnimationsOnPerformanceMode } from "libs/ui/react/utils/performance";

import "@shared/styles/colors.css";
import "./reset.css";

disableAnimationsOnPerformanceMode();

const { js, css, html } = window.__SLU_WIDGET;

if (html) {
  document.body.innerHTML = html;
}

if (css) {
  const style = document.createElement("style");
  style.textContent = css;
  document.head.appendChild(style);
}

if (js) {
  const script = document.createElement("script");
  script.type = "module";
  script.textContent = js;
  document.head.appendChild(script);
}
