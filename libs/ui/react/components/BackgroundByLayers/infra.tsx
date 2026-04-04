import { cx } from "libs/ui/react/utils/styling";

import cs from "./infra.module.css";

export function BackgroundByLayersV2() {
  let background = (
    <div className={cx(cs.background, "bg-layers")}>
      {Array.from({ length: 10 }, (_, index) => <div key={index} className={cx(cs.layer, `bg-layer-${index + 1}`)} />)}
    </div>
  );

  return background;
}
