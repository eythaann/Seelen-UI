import type { WmNode } from "@seelen-ui/lib/types";
import { FileIcon } from "@shared/components/Icon";
import { cx } from "@shared/styles";

import { $open_apps, $overlay_visible } from "../../../shared/state/mod.ts";
import { Leaf } from "./leaf.tsx";

interface Props {
  node: WmNode;
}

export function Stack({ node }: Props) {
  return (
    <div
      style={{
        flexGrow: node.growFactor,
      }}
      className={cx("wm-container", "wm-stack")}
    >
      {node.windows.length > 1 && (
        <div
          className="wm-stack-bar"
          data-allow-mouse-events={$overlay_visible.value}
        >
          {node.windows.map((handle) => {
            const info = $open_apps.value.find((app) => app.windows.some((w) => w.handle === handle));

            return (
              <div
                key={handle}
                className={cx("wm-stack-bar-item", {
                  "wm-stack-bar-item-active": handle === node.active,
                })}
                data-allow-mouse-events={$overlay_visible.value}
              >
                <FileIcon
                  path={info?.path}
                  umid={info?.umid}
                  className="wm-stack-bar-item-icon"
                  data-allow-mouse-events={$overlay_visible.value}
                />
                <span
                  className="wm-stack-bar-item-title"
                  data-allow-mouse-events={$overlay_visible.value}
                >
                  {info?.windows.find((w) => w.handle === handle)?.title ||
                    `0x${handle.toString(16)}`}
                </span>
              </div>
            );
          })}
        </div>
      )}
      {node.active && <Leaf hwnd={node.active} />}
    </div>
  );
}
