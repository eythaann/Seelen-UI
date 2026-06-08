import { BorderSettings } from "../../border/infra.tsx";

import { WmAnimationsSettings } from "./Animations.tsx";
import { GlobalPaddings } from "./GlobalPaddings.tsx";
import { OthersConfigs } from "./Others.tsx";
import { LayoutSelector } from "./LayoutSelector.tsx";

export function WindowManagerSettings() {
  return (
    <>
      <LayoutSelector />
      <GlobalPaddings />
      <BorderSettings />
      <WmAnimationsSettings />
      <OthersConfigs />
    </>
  );
}
