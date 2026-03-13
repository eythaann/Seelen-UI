import { PerformanceMode } from "@seelen-ui/types";
import { invoke, SeelenCommand, SeelenEvent, subscribe } from "../../handlers/mod.ts";

export async function disableAnimationsOnPerformanceMode(): Promise<void> {
  const initial = await invoke(SeelenCommand.StateGetPerformanceMode);
  setDisableAnimations(initial);
  subscribe(SeelenEvent.StatePerformanceModeChanged, (e) => {
    setDisableAnimations(e.payload);
  });
}

function setDisableAnimations(mode: PerformanceMode): void {
  const root = document.documentElement;
  if (mode === PerformanceMode.Extreme) {
    root.setAttribute("data-animations-off", "");
  } else {
    root.removeAttribute("data-animations-off");
  }
}
