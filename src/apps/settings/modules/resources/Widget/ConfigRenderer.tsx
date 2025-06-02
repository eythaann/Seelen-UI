import { WidgetSettingsDeclarationList, WsdGroupEntry, WsdItem } from '@seelen-ui/lib/types';
import { ResourceText } from '@shared/components/ResourceText';
import { ColorPicker, Input, InputNumber, Select, Slider, Switch } from 'antd';

import {
  SettingsGroup,
  SettingsOption,
  SettingsSubGroup,
} from 'src/apps/settings/components/SettingsBox';

interface Props {
  // settings definitions
  definitions: WidgetSettingsDeclarationList;
  // config values set by the user
  config: Record<string, any>;
  // callback to update the config
  onConfigChange: (key: string, value: any) => void;
  // whether the widget is being configured by monitor
  isByMonitor?: boolean;
}

export function RenderBySettingsDeclaration({ definitions, config, onConfigChange }: Props) {
  return definitions.map(({ group }, idx) => {
    return (
      <SettingsGroup key={idx}>
        {group.map((entry, idx) => (
          <WSGroupEntry key={idx} entry={entry} config={config} onConfigChange={onConfigChange} />
        ))}
      </SettingsGroup>
    );
  });
}

// ================================================

interface WSGroupEntryProps {
  entry: WsdGroupEntry;
  config: Record<string, any>;
  onConfigChange: (key: string, value: any) => void;
}

function WSGroupEntry(props: WSGroupEntryProps) {
  const { config, onConfigChange } = props;

  if ('subgroup' in props.entry) {
    const {
      subgroup: { header, content },
    } = props.entry;

    return (
      <SettingsSubGroup
        label={
          header ? <WSItem def={header} config={config} onConfigChange={onConfigChange} /> : ''
        }
      >
        {content.map((entry, idx) => {
          return (
            <WSGroupEntry key={idx} entry={entry} config={config} onConfigChange={onConfigChange} />
          );
        })}
      </SettingsSubGroup>
    );
  }

  const { config: definition } = props.entry;
  return <WSItem def={definition} config={config} onConfigChange={onConfigChange} />;
}

// ================================================

interface WSItemProps {
  def: WsdItem;
  config: Record<string, any>;
  onConfigChange: (key: string, value: any) => void;
}

function WSItem({ def, config, onConfigChange }: WSItemProps) {
  let action: React.ReactNode = null;

  const commonProps = {
    defaultValue: def.defaultValue as any,
    value: config[def.key],
    onChange: (value: any) => onConfigChange(def.key, value),
  };

  if (def.type === 'switch') {
    action = <Switch {...commonProps} />;
  } else if (def.type === 'select') {
    action = <Select {...commonProps} options={def.options} />;
  } else if (def.type === 'input-text') {
    action = <Input {...commonProps} onChange={(e) => onConfigChange(def.key, e.target.value)} />;
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
