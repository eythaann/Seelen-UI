import type { ThemeId, ThemeVariableDefinition } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { Button, ColorPicker, Input, InputNumber, Select, Slider, Space, Tooltip } from "antd";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";

import { SettingsOption } from "../../../../components/SettingsBox/index.tsx";
import { CSS_UNITS } from "../constants.ts";
import { useThemeVariable } from "../hooks/useThemeVariable.ts";

export interface ThemeSettingProps {
  themeId: ThemeId;
  definition: ThemeVariableDefinition;
}

export function ThemeSetting({ themeId, definition }: ThemeSettingProps) {
  const { value: userStoredValue, onChange, onReset } = useThemeVariable(themeId, definition.name);
  const { t } = useTranslation();

  const input = renderInput(definition, userStoredValue, onChange, onReset);

  return (
    <SettingsOption
      label={<ResourceText text={definition.label} />}
      tip={definition.tip ? <ResourceText text={definition.tip} /> : undefined}
      description={definition.description ? <ResourceText text={definition.description} /> : undefined}
      action={
        <Space.Compact>
          {input}
          <Tooltip title={t("reset_to_default")}>
            <Button onClick={onReset}>
              <Icon iconName="BiReset" />
            </Button>
          </Tooltip>
        </Space.Compact>
      }
    />
  );
}

function renderInput(
  definition: ThemeVariableDefinition,
  userStoredValue: string | undefined,
  onChange: (value: string) => void,
  onReset: () => void,
): ReactNode {
  if (definition.options) {
    return (
      <Select
        options={definition.options.map((value) => ({ value: String(value) }))}
        value={userStoredValue}
        defaultValue={`${definition.initialValue}`}
        onChange={onChange}
      />
    );
  }

  const { min, max, step } = definition;

  switch (definition.syntax) {
    case "<color>": {
      const value = userStoredValue || definition.initialValue;
      return (
        <ColorPicker
          showText
          value={value}
          onChangeComplete={(color) => onChange(color.toHexString())}
        />
      );
    }

    case "<length-percentage>": {
      const numericValue = userStoredValue ? parseFloat(userStoredValue) : definition.initialValue;
      const unit = userStoredValue?.replace(/[\d.]+/, "") || definition.initialValueUnit;

      return (
        <Space.Compact>
          <InputNumber
            value={numericValue}
            onChange={(newValue) => {
              if (newValue == null) {
                onReset();
                return;
              }
              onChange(`${newValue}${unit}`);
            }}
            min={min || undefined}
            max={max || undefined}
            step={step || undefined}
          />
          <Select
            options={CSS_UNITS.map((unit) => ({ value: unit }))}
            style={{ width: 60, minWidth: 60 }}
            value={unit}
            onChange={(newUnit) => onChange(`${numericValue}${newUnit}`)}
          />
        </Space.Compact>
      );
    }

    case "<number>": {
      const value = userStoredValue ? parseFloat(userStoredValue) : definition.initialValue;

      const handleChange = (newValue: number | null) => {
        if (newValue == null) {
          onReset();
          return;
        }
        onChange(`${newValue}`);
      };

      if (step != null) {
        return (
          <Slider
            style={{ flex: 1, minWidth: 120 }}
            value={value}
            onChange={handleChange}
            min={min || undefined}
            max={max || undefined}
            step={step || undefined}
          />
        );
      }

      return (
        <InputNumber
          value={value}
          onChange={handleChange}
          min={min || undefined}
          max={max || undefined}
          step={step || undefined}
        />
      );
    }

    case "<url>": {
      const handleSelectFile = async () => {
        const selected = await open({
          multiple: false,
          directory: false,
          filters: [
            {
              name: "Image",
              extensions: ["jpg", "jpeg", "png", "gif", "webp", "svg"],
            },
          ],
        });

        if (selected) {
          onChange(convertFileSrc(selected));
        }
      };

      return (
        <Space.Compact style={{ flex: 1 }}>
          <Input readOnly value={userStoredValue} placeholder="Select a file..." />
          <Button onClick={handleSelectFile}>
            <Icon iconName="FaFolderOpen" />
          </Button>
        </Space.Compact>
      );
    }

    case "<string>": {
      const value = userStoredValue ?? definition.initialValue;
      return (
        <Input
          value={value}
          onChange={(e) => onChange(e.currentTarget.value)}
          minLength={min || undefined}
          maxLength={max || undefined}
        />
      );
    }

    default: {
      // @ts-expect-error should never happen
      definition.syntax;
      return null;
    }
  }
}
