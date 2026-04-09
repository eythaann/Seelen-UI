import { _invoke } from "./_tauri";
import type { Alignment, WidgetId, WidgetTriggerPayload } from "@seelen-ui/lib/types";

let timeoutRef: ReturnType<typeof setTimeout> | null = null;
function showTooltip(text: string | undefined) {
  if (timeoutRef) {
    clearTimeout(timeoutRef);
  }

  if (!text) {
    return;
  }

  timeoutRef = setTimeout(() => {
    const payload: WidgetTriggerPayload = {
      id: "@seelen/tooltip" as WidgetId,
      alignX: "Center" as Alignment,
      customArgs: { text, show: true },
    };
    _invoke("trigger_widget", { payload });
  }, 400);
}

function hideTooltip() {
  if (timeoutRef) {
    clearTimeout(timeoutRef);
  }

  const payload: WidgetTriggerPayload = {
    id: "@seelen/tooltip" as WidgetId,
    customArgs: { show: false },
  };
  _invoke("trigger_widget", { payload });
}

let lastShownOn: any = null;
document.addEventListener(
  "pointerenter",
  (e) => {
    const tooltip = (e.target as HTMLElement | null)?.dataset?.tooltip;
    if (!tooltip) {
      return;
    }

    lastShownOn = e.target;
    showTooltip(tooltip);
  },
  true,
);

document.addEventListener(
  "pointerleave",
  (e) => {
    if (lastShownOn !== e.target) {
      return;
    }
    hideTooltip();
  },
  true,
);

globalThis.addEventListener("blur", () => {
  hideTooltip();
  lastShownOn = null;
});

// UX for buttons
document.addEventListener("keydown", (e: KeyboardEvent) => {
  if (e.defaultPrevented) return;
  if (e.key !== "Enter" && e.key !== " ") return;

  const target = e.target as HTMLElement;
  if (target.getAttribute("role") !== "button") return;

  target.dataset["ux-active"] = "true";
});

document.addEventListener("keyup", (e: KeyboardEvent) => {
  if (e.defaultPrevented) return;
  if (e.key !== "Enter" && e.key !== " ") return;

  const target = e.target as HTMLElement;
  if (target.getAttribute("role") !== "button") return;

  if (target.dataset["ux-active"] !== "true") {
    target.dataset["ux-active"] = "false";

    if ("click" in target) {
      target.click();
    }
  }
});
