import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { PerformanceMode } from "@seelen-ui/lib/types";

export async function disableAnimationsOnPerformanceMode() {
  let initial = await invoke(SeelenCommand.StateGetPerformanceMode);
  setDisableAnimations(initial);
  subscribe(SeelenEvent.StatePerformanceModeChanged, (e) => {
    setDisableAnimations(e.payload);
  });
}

function setDisableAnimations(_mode: PerformanceMode) {
  // we gonna do nothing for now in meantime we refactor and remove all atnd code.
  /* if (mode === "Extreme") {
    let style = document.createElement("style");
    style.id = DISABLE_ANIMATIONS_ID;
    style.appendChild(document.createTextNode(DISABLE_ANIMATIONS_CSS));
    document.head.appendChild(style);
  } else {
    document.getElementById(DISABLE_ANIMATIONS_ID)?.remove();
  } */
}

const _DISABLE_ANIMATIONS_ID = "force-disable-animations";
const _DISABLE_ANIMATIONS_CSS = `
* {
  transition: none !important;
  animation: none !important;
}
`;
