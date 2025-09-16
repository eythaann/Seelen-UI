import { cx } from "@shared/styles";

import { $focused_app, $settings } from "../../../shared/state/mod";
// import { ReservedContainer } from './reserved';

interface Props {
  hwnd: number;
  growFactor?: number;
}

export function Leaf({ hwnd, growFactor }: Props) {
  const isFocused = $focused_app.value.hwnd === hwnd;
  return (
    <div
      data-hwnd={hwnd}
      style={{
        flexGrow: growFactor,
      }}
      className={cx("wm-container", "wm-leaf", {
        "wm-leaf-focused": isFocused,
        "wm-leaf-with-borders": $settings.value.border.enabled,
      })}
    >
      {/* {!!reservation && isFocused && <ReservedContainer reservation={reservation} />} */}
    </div>
  );
}
