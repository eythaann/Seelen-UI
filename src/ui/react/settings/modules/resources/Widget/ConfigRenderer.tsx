import type { WidgetConfigDefinition, WidgetSettingItem, WidgetSettingsDeclarationList } from "@seelen-ui/lib/types";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { ColorPicker, Input, InputNumber, Select, Slider, Switch } from "antd";
import type { ReactNode } from "react";
import { useMemo } from "react";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../../components/SettingsBox/index.tsx";

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
  return (
    <>
      {definitions.map((definition, idx) => (
        <WidgetConfigDefinition
          key={idx}
          definition={definition}
          values={values}
          onConfigChange={onConfigChange}
          nestLevel={0}
        />
      ))}
    </>
  );
}

// ================================================

interface WidgetConfigDefinitionProps {
  definition: WidgetConfigDefinition;
  values: Record<string, any>;
  onConfigChange: (key: string, value: any) => void;
  nestLevel: number;
}

function WidgetConfigDefinition({
  definition,
  values,
  onConfigChange,
  nestLevel,
}: WidgetConfigDefinitionProps) {
  const content = renderContent(definition, values, onConfigChange, nestLevel);

  return nestLevel === 0 ? <SettingsGroup>{content}</SettingsGroup> : content;
}

function renderContent(
  definition: WidgetConfigDefinition,
  values: Record<string, any>,
  onConfigChange: (key: string, value: any) => void,
  nestLevel: number,
): ReactNode {
  // Check if it's a group (has "group" property)
  if ("group" in definition) {
    return (
      <SettingsSubGroup label={<ResourceText text={definition.group.label} />}>
        {definition.group.items.map((item, idx) => (
          <WidgetConfigDefinition
            key={idx}
            definition={item}
            values={values}
            onConfigChange={onConfigChange}
            nestLevel={nestLevel + 1}
          />
        ))}
      </SettingsSubGroup>
    );
  }

  // It's a setting item
  return <WidgetSettingItemRenderer def={definition} values={values} onConfigChange={onConfigChange} />;
}

// ================================================

interface WidgetSettingItemRendererProps {
  def: WidgetSettingItem;
  values: Record<string, any>;
  onConfigChange: (key: string, value: any) => void;
}

function WidgetSettingItemRenderer({ def, values, onConfigChange }: WidgetSettingItemRendererProps) {
  // Check if all dependencies are met
  const isDependencyMet = useMemo(() => {
    if (!def.dependencies || def.dependencies.length === 0) {
      return true;
    }
    return def.dependencies.every((depKey) => !!values[depKey]);
  }, [def.dependencies, values]);

  if (!isDependencyMet) {
    return null;
  }

  const action = renderInput(def, values, onConfigChange);

  return (
    <SettingsOption
      label={<ResourceText text={def.label} />}
      tip={def.tip ? <ResourceText text={def.tip} /> : undefined}
      description={def.description ? <ResourceText text={def.description} /> : undefined}
      action={action}
    />
  );
}

function renderInput(
  def: WidgetSettingItem,
  values: Record<string, any>,
  onConfigChange: (key: string, value: any) => void,
): ReactNode {
  const commonProps = {
    defaultValue: def.defaultValue as any,
    value: values[def.key],
    onChange: (value: any) => onConfigChange(def.key, value),
  };

  switch (def.type) {
    case "switch": {
      return <Switch {...commonProps} />;
    }

    case "select": {
      // Convert WidgetSelectOption[] to Ant Design Select options format
      const options = def.options.map((opt) => ({
        label: <ResourceText text={opt.label} />,
        value: opt.value,
      }));

      return <Select {...commonProps} options={options} />;
    }

    case "input-text": {
      const textProps = {
        ...commonProps,
        minLength: def.minLength ?? undefined,
        maxLength: def.maxLength ?? undefined,
        onChange: (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) =>
          onConfigChange(def.key, e.currentTarget.value),
      };

      if (def.multiline) {
        return <Input.TextArea {...textProps} />;
      }

      return <Input {...textProps} />;
    }

    case "input-number": {
      return (
        <InputNumber
          {...commonProps}
          min={def.min ?? undefined}
          max={def.max ?? undefined}
          step={def.step ?? undefined}
        />
      );
    }

    case "range": {
      return (
        <Slider
          {...commonProps}
          style={{ width: "200px" }}
          min={def.min ?? undefined}
          max={def.max ?? undefined}
          step={def.step ?? undefined}
        />
      );
    }

    case "color": {
      return (
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

    default: {
      // @ts-expect-error should never happen
      def.type;
      return null;
    }
  }
}
