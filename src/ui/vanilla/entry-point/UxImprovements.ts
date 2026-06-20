import { _invoke } from "./_tauri";
import { Alignment, type WidgetId, type WidgetTriggerPayload } from "@seelen-ui/lib/types";

function alignment(value: any): Alignment {
  return Object.values(Alignment).includes(value) ? value : Alignment.Start;
}

let timeoutRef: ReturnType<typeof setTimeout> | null = null;
let tooltipVisible = false;
let lastShownOn: any = null;

function showTooltip(text: string, alignX: Alignment, alignY: Alignment) {
  if (timeoutRef) {
    clearTimeout(timeoutRef);
  }

  timeoutRef = setTimeout(() => {
    tooltipVisible = true;
    const payload: WidgetTriggerPayload = {
      id: "@seelen/tooltip" as WidgetId,
      alignX,
      alignY,
      customArgs: { text, show: true },
    };
    _invoke("trigger_widget", { payload });
  }, 400);
}

function hideTooltip() {
  if (timeoutRef) {
    clearTimeout(timeoutRef);
    timeoutRef = null;
  }

  lastShownOn = null;

  if (!tooltipVisible) {
    return;
  }

  tooltipVisible = false;
  const payload: WidgetTriggerPayload = {
    id: "@seelen/tooltip" as WidgetId,
    customArgs: { show: false },
  };
  _invoke("trigger_widget", { payload });
}

document.addEventListener(
  "pointerenter",
  (e) => {
    if (!window.__SLU_WIDGET_INSTANCE?.isReady) {
      return;
    }

    const target = e.target;
    if (!target || !(target instanceof HTMLElement)) {
      return;
    }

    const tooltip = target.dataset.tooltip || target.title || target.getAttribute("aria-label");
    if (!tooltip) {
      return;
    }

    if (target.title) {
      target.removeAttribute("title");
      target.dataset["tooltip"] = tooltip;
    }

    lastShownOn = e.target;
    showTooltip(
      tooltip,
      alignment(target.dataset.tooltipAlignX),
      alignment(target.dataset.tooltipAlignY),
    );
  },
  true,
);

document.addEventListener(
  "pointerleave",
  (e) => {
    if (lastShownOn === e.target) {
      hideTooltip();
    }
  },
  true,
);

globalThis.addEventListener("blur", () => {
  if (lastShownOn) {
    hideTooltip();
  }
});

/*
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
*/
