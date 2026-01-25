import type { WidgetId } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button, Select, Space } from "antd";

import { patchWidgetInstanceConfig, removeWidgetInstance } from "./application.ts";

interface InstanceSelectorProps {
  widgetId: WidgetId;
  selected: string | null;
  onSelect: (value: string | null) => void;
  options: {
    value: string;
    label: string;
  }[];
}

export function WidgetInstanceSelector(
  { widgetId, options, selected, onSelect }: InstanceSelectorProps,
) {
  const onInstanceCreated = () => {
    const instanceId = crypto.randomUUID();
    patchWidgetInstanceConfig(widgetId, instanceId, {});
    onSelect(instanceId);
  };

  const onInstanceDeleted = () => {
    if (selected) {
      const idx = options.findIndex((t) => t.value === selected);
      const newIdx = idx === 0 ? idx + 1 : idx - 1;
      onSelect(options[newIdx]?.value || null);
      removeWidgetInstance(widgetId, selected);
    }
  };

  return (
    <Space.Compact>
      <Select
        style={{ width: 300 }}
        value={selected}
        onSelect={onSelect}
        options={options}
        allowClear
        onClear={() => onSelect(null)}
        placeholder="-"
      />
      <Button onClick={onInstanceCreated}>
        <Icon iconName="FaPlus" />
      </Button>
      <Button onClick={onInstanceDeleted}>
        <Icon iconName="IoTrash" />
      </Button>
    </Space.Compact>
  );
}
