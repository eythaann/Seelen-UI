import { WidgetId } from '@seelen-ui/lib/types';
import { Icon } from '@shared/components/Icon';
import { Button, Select, Space } from 'antd';
import { useDispatch } from 'react-redux';

import { RootActions } from '../../shared/store/app/reducer';

interface InstanceSelectorProps {
  widgetId: WidgetId;
  selected: string | null;
  onSelect: (value: string | null) => void;
  options: {
    value: string;
    label: string;
  }[];
}

export function WidgetInstanceSelector({ widgetId, options, selected, onSelect }: InstanceSelectorProps) {
  const d = useDispatch();

  const onInstanceCreated = () => {
    const instanceId = crypto.randomUUID();
    d(RootActions.patchWidgetInstanceConfig({ widgetId, instanceId, config: {} }));
    onSelect(instanceId);
  };

  const onInstanceDeleted = () => {
    if (selected) {
      const idx = options.findIndex((t) => t.value === selected);
      const newIdx = idx === 0 ? idx + 1 : idx - 1;
      onSelect(options[newIdx]?.value || null);
      d(RootActions.removeWidgetInstance({ widgetId, instanceId: selected }));
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
