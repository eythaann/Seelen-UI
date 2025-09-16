import { effect, useSignalEffect } from "@preact/signals";
import { WmNodeKind } from "@seelen-ui/lib";
import { cx } from "@shared/styles";

import { requestPositioningOfLeaves } from "../application";

import { Node } from "../domain";

import { $force_repositioning, $layout, $overlay_visible, $settings } from "../../shared/state/mod";
import { NodeUtils } from "../../shared/utils";
import { Leaf } from "./containers/leaf";
import { Stack } from "./containers/stack";

import "./index.css";

effect(() => {
  document.body.style.opacity = $overlay_visible.value ? "1" : "0";
});

export function Layout() {
  useSignalEffect(() => {
    $settings.value;
    $layout.value;
    $force_repositioning.value;
    requestPositioningOfLeaves();
  });

  if (!$layout.value) {
    return null;
  }

  return <Container node={$layout.value} />;
}

export function Container({ node }: { node: Node }) {
  if (NodeUtils.isEmpty(node)) {
    return null;
  }

  if (node.type === WmNodeKind.Stack) {
    return <Stack node={node} />;
  }

  if (node.type === WmNodeKind.Leaf && node.active) {
    return <Leaf hwnd={node.active} growFactor={node.growFactor} />;
  }

  if (
    node.type === WmNodeKind.Horizontal || node.type === WmNodeKind.Vertical
  ) {
    return (
      <div
        style={{
          flexGrow: node.growFactor,
        }}
        className={cx("wm-container", `wm-${node.type.toLowerCase()}`)}
      >
        {node.children.map((child, idx) => <Container key={idx} node={child} />)}
      </div>
    );
  }

  return null;
}
