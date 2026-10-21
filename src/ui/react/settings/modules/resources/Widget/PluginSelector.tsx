import type { PluginId, WidgetId } from "@seelen-ui/lib/types";
import { Select } from "antd";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { plugins } from "../../../state/resources";

interface Props {
  /** widget targeted by the selectable plugins */
  target: WidgetId;
  value?: PluginId;
  onSelect: (value: PluginId) => void;
}

export function PluginSelect({ target, value, onSelect }: Props) {
  const options = plugins.value.filter((plugin) => plugin.target === target);

  return (
    <Select
      style={{ width: "200px" }}
      value={value}
      options={options.map((plugin) => ({
        key: plugin.id,
        label: <ResourceText text={plugin.metadata.displayName} />,
        value: plugin.id,
      }))}
      onSelect={onSelect}
    />
  );
}
