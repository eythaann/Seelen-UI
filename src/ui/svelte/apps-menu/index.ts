import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import type { PhysicalMonitor, WidgetTriggerPayload } from "@seelen-ui/lib/types";
import { getRootContainer } from "libs/ui/react/utils/index.ts";
import { mount } from "svelte";
import { StartDisplayMode } from "./constants";
import { getAppsMenuRect, resolveTargetMonitor } from "./state/geometry";

import "@seelen-ui/lib/styles/reset.css";
import "./loading.css";

const widget = Widget.getCurrent();
const root = getRootContainer();

const loadingShell = document.createElement("div");
loadingShell.className = "apps-menu apps-menu-loading";
loadingShell.dataset.fullscreen = "false";

const loadingIndicator = document.createElement("div");
loadingIndicator.className = "apps-menu-loading-indicator";
loadingIndicator.setAttribute("aria-hidden", "true");
loadingShell.append(loadingIndicator);
root.append(loadingShell);

await widget.init({ hideOnFocusLoss: true });
await widget.window.setFocusable(true);

type PositioningModule = typeof import("./state/positioning.svelte.ts");

let positioningModule: PositioningModule | null = null;
let contentLoadPromise: Promise<void> | null = null;
let lastTrigger: WidgetTriggerPayload | null = null;

async function readInitialDisplayMode(): Promise<StartDisplayMode> {
  try {
    const stored = JSON.parse(
      await invoke(SeelenCommand.ReadFile, { filename: "display_mode.json" }),
    );
    return stored === StartDisplayMode.Fullscreen ? StartDisplayMode.Fullscreen : StartDisplayMode.Normal;
  } catch {
    return StartDisplayMode.Normal;
  }
}

async function positionLoadingShell(args: WidgetTriggerPayload): Promise<void> {
  const [monitors, displayMode] = await Promise.all([
    invoke(SeelenCommand.SystemGetMonitors) as Promise<PhysicalMonitor[]>,
    readInitialDisplayMode(),
  ]);
  const targetMonitor = resolveTargetMonitor(monitors, args.monitorId);
  if (!targetMonitor) return;

  loadingShell.dataset.fullscreen = String(displayMode === StartDisplayMode.Fullscreen);
  await widget.setPosition(getAppsMenuRect(targetMonitor, displayMode));
}

function ensureContentLoaded(): Promise<void> {
  if (contentLoadPromise) return contentLoadPromise;

  contentLoadPromise = (async () => {
    const [{ default: App }, positioning] = await Promise.all([
      import("./App.svelte"),
      import("./state/positioning.svelte.ts"),
    ]);
    positioningModule = positioning;

    if (lastTrigger) {
      await positioning.prepareForTrigger(lastTrigger.monitorId);
    }

    loadingShell.remove();
    mount(App, { target: root });
  })().catch((error) => {
    contentLoadPromise = null;
    console.error("Failed to load apps menu content", error);
  });

  return contentLoadPromise;
}

widget.onTrigger(async (args) => {
  const visible = await widget.window.isVisible();
  if (visible) {
    await widget.hide();
  } else if (positioningModule) {
    await positioningModule.onTriggered(args.monitorId);
  } else {
    lastTrigger = args;
    try {
      await positionLoadingShell(args);
    } catch (error) {
      console.error("Failed to position apps menu loading shell", error);
    }
    await widget.show();
    await widget.focus();
    void ensureContentLoaded();
  }
});

await widget.ready({ show: false });
void ensureContentLoaded();
