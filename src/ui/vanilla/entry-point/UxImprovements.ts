import { _invoke } from "./_tauri";
import { Alignment, type WidgetId, type WidgetTriggerPayload } from "@seelen-ui/lib/types";

function alignment(value: any): Alignment {
  return Object.values(Alignment).includes(value) ? value : Alignment.Start;
}

let timeoutRef: ReturnType<typeof setTimeout> | null = null;
let tooltipVisible = false;
let lastShownOn: HTMLElement | null = null;
let tooltipObserver: MutationObserver | null = null;

function getTooltipText(target: HTMLElement): string | null {
  const isRange = target instanceof HTMLInputElement && target.type === "range" && !!target.dataset.skin;
  return isRange ? target.value : target.dataset.tooltip || target.title || target.getAttribute("aria-label");
}

// `title` is moved into `data-tooltip` so every code path (CSS, MutationObserver) can
// rely on a single attribute; re-run this whenever `title` may have been set again.
function migrateTitleToDataset(target: HTMLElement) {
  if (target.title) {
    const text = target.title;
    target.removeAttribute("title");
    target.dataset["tooltip"] = text;
  }
}

function updateTooltipText(target: HTMLElement) {
  migrateTitleToDataset(target);
  const tooltip = getTooltipText(target);
  if (tooltip) {
    showTooltip(
      tooltip,
      alignment(target.dataset.tooltipAlignX),
      alignment(target.dataset.tooltipAlignY),
      true,
    );
  }
}

function handleTooltipInput(e: Event) {
  updateTooltipText(e.currentTarget as HTMLElement);
}

function setTooltipParentElement(element: HTMLElement) {
  if (lastShownOn) {
    clearTooltipParentElement();
  }
  lastShownOn = element;
  lastShownOn.addEventListener("pointerleave", hideTooltip, { once: true });
  // `value` changes on inputs (e.g. dragging a range slider) only touch the IDL
  // property, never the `value` attribute, so a MutationObserver can't see them.
  lastShownOn.addEventListener("input", handleTooltipInput);

  tooltipObserver = new MutationObserver(() => updateTooltipText(element));
  tooltipObserver.observe(element, {
    attributes: true,
    attributeFilter: ["title", "data-tooltip", "aria-label"],
  });
}

function clearTooltipParentElement() {
  lastShownOn?.removeEventListener("pointerleave", hideTooltip);
  lastShownOn?.removeEventListener("input", handleTooltipInput);
  lastShownOn = null;
  tooltipObserver?.disconnect();
  tooltipObserver = null;
}

function showTooltip(text: string, alignX: Alignment, alignY: Alignment, immediate = false) {
  if (timeoutRef) {
    clearTimeout(timeoutRef);
    timeoutRef = null;
  }

  const run = () => {
    tooltipVisible = true;
    const payload: WidgetTriggerPayload = {
      id: "@seelen/tooltip" as WidgetId,
      alignX,
      alignY,
      customArgs: { text, show: true },
    };
    _invoke("trigger_widget", { payload });
  };

  if (immediate) {
    run();
  } else {
    timeoutRef = setTimeout(run, 400);
  }
}

function hideTooltip() {
  if (timeoutRef) {
    clearTimeout(timeoutRef);
    timeoutRef = null;
  }

  clearTooltipParentElement();

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

    const tooltip = getTooltipText(target);
    if (!tooltip) {
      return;
    }

    migrateTitleToDataset(target);

    setTooltipParentElement(target);
    showTooltip(
      tooltip,
      alignment(target.dataset.tooltipAlignX),
      alignment(target.dataset.tooltipAlignY),
    );
  },
  true,
);

// UX for sliders

document.addEventListener(
  "wheel",
  (e: WheelEvent) => {
    const target = e.target;
    if (
      !target ||
      !(target instanceof HTMLInputElement) ||
      target.type !== "range" ||
      !target.dataset.skin
    ) {
      return;
    }

    if (target.disabled) {
      return;
    }

    e.preventDefault();

    const step = Number(target.step) || 1;
    const min = target.min !== "" ? Number(target.min) : 0;
    const max = target.max !== "" ? Number(target.max) : 100;
    const direction = e.deltaY < 0 ? 1 : -1;

    const value = Math.min(max, Math.max(min, Number(target.value) + direction * step));
    if (value === Number(target.value)) {
      return;
    }

    target.value = String(value);
    target.dispatchEvent(new Event("input", { bubbles: true }));
    target.dispatchEvent(new Event("change", { bubbles: true }));
  },
  { passive: false, capture: true },
);

// UX for buttons

/*
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
