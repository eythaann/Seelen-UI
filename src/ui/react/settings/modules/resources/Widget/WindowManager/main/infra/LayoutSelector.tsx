import { getWmConfig, setWmDefaultLayout } from "../../application";
import { plugins } from "src/ui/react/settings/state/resources";
import { SeelenWindowManagerWidgetId } from "@seelen-ui/lib";
import { TwmNodeKind, type TwmPluginNode } from "@seelen-ui/lib/types";
import { SettingsGroup, SettingsOption } from "src/ui/react/settings/components/SettingsBox";
import { Select } from "antd";
import { ResourceText } from "libs/ui/react/components/ResourceText";
import { useTranslation } from "react-i18next";

import cs from "./LayoutSelector.module.css";

export function LayoutSelector() {
  const wmSettings = getWmConfig();
  const defaultLayout = wmSettings.defaultLayout;

  const layouts = plugins.value.filter((plugin) => plugin.target === SeelenWindowManagerWidgetId);
  const usingLayout = layouts.find((plugin) => plugin.id === defaultLayout);

  const { t } = useTranslation();

  return (
    <SettingsGroup>
      <SettingsOption
        label={t("wm.layout")}
        action={
          <Select
            style={{ width: "200px" }}
            value={defaultLayout}
            options={layouts.map((layout) => ({
              key: layout.id,
              label: <ResourceText text={layout.metadata.displayName} />,
              value: layout.id,
            }))}
            onSelect={(value) => setWmDefaultLayout(value)}
          />
        }
        description={usingLayout ? <ResourceText text={usingLayout.metadata.description} /> : undefined}
      />
      {
        /* <div class={cs.monitor}>
        <div class={cs.screen}>
          {usingLayout && <TwmNode node={(usingLayout.plugin as TwmPlugin).structure} />}
        </div>
      </div> */
      }
    </SettingsGroup>
  );
}

function TwmNode({ node }: { node: TwmPluginNode | null }) {
  if (!node) {
    return <div className={cs.float}></div>;
  }

  if (node.kind === TwmNodeKind.Horizontal || node.kind === TwmNodeKind.Vertical) {
    return (
      <div
        class={cs.container}
        style={{
          display: "flex",
          flexDirection: node.kind === TwmNodeKind.Horizontal ? "row" : "column",
          flexGrow: node.growFactor,
        }}
      >
        {node.children.map((node, idx) => <TwmNode key={idx} node={node} />)}
      </div>
    );
  }

  return <div className={cs.leaf} style={{ flexGrow: node.growFactor }}></div>;
}
