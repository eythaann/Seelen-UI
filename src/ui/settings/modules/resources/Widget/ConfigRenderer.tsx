import { WidgetSettingsDeclarationList, WsdGroupEntry, WsdItem } from '@seelen-ui/lib/types';
import { ResourceText } from '@shared/components/ResourceText';
import { ColorPicker, Input, InputNumber, Select, Slider, Switch } from 'antd';

import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../../components/SettingsBox';

interface Props {
  // settings definitions
  definitions: WidgetSettingsDeclarationList;
  // config values set by the user
  values: Record<string, any>;
  // callback to update the config
  onConfigChange: (key: string, value: any) => void;
  // whether the widget is being configured by monitor
  isByMonitor?: boolean;
}

export function RenderBySettingsDeclaration({ definitions, values, onConfigChange }: Props) {
  return definitions.map(({ group }, idx) => {
    return (
      <SettingsGroup key={idx}>
        {group.map((entry, idx) => (
          <WSGroupEntry key={idx} entry={entry} values={values} onConfigChange={onConfigChange} />
        ))}
      </SettingsGroup>
    );
  });
}

// ================================================

interface WSGroupEntryProps {
  entry: WsdGroupEntry;
  values: Record<string, any>;
  onConfigChange: (key: string, value: any) => void;
}

function WSGroupEntry(props: WSGroupEntryProps) {
  const { entry, values, onConfigChange } = props;

  if (entry.children.length > 0) {
    return (
      <SettingsSubGroup
        label={<WSItem def={entry.config} values={values} onConfigChange={onConfigChange} />}
      >
        {entry.children.map((entry, idx) => (
          <WSGroupEntry key={idx} entry={entry} values={values} onConfigChange={onConfigChange} />
        ))}
      </SettingsSubGroup>
    );
  }

  return <WSItem def={entry.config} values={values} onConfigChange={onConfigChange} />;
}

// ================================================

interface WSItemProps {
  def: WsdItem;
  values: Record<string, any>;
  onConfigChange: (key: string, value: any) => void;
}

function WSItem({ def, values, onConfigChange }: WSItemProps) {
  let action: React.ReactNode = null;

  const commonProps = {
    defaultValue: def.defaultValue as any,
    value: values[def.key],
    onChange: (value: any) => onConfigChange(def.key, value),
  };

  if (def.type === 'switch') {
    action = <Switch {...commonProps} />;
  } else if (def.type === 'select') {
    action = <Select {...commonProps} options={def.options} />;
  } else if (def.type === 'input-text') {
    action = (
      <Input {...commonProps} onChange={(e) => onConfigChange(def.key, e.currentTarget.value)} />
    );
  } else if (def.type === 'input-number') {
    action = <InputNumber {...commonProps} />;
  } else if (def.type === 'range') {
    action = (
      <Slider
        {...commonProps}
        style={{ width: '200px' }}
        step={def.step}
        min={def.from}
        max={def.to}
      />
    );
  } else if (def.type === 'color') {
    action = (
      <ColorPicker
        {...commonProps}
        disabledAlpha={!def.allowAlpha}
        onChange={undefined}
        onChangeComplete={(v) => {
          onConfigChange(def.key, v.toHexString());
        }}
      />
    );
  }

  return (
    <SettingsOption>
      <span>
        <ResourceText text={def.label} />
      </span>
      {action}
    </SettingsOption>
  );
}
