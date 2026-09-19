import type { ThemeId, ThemeVariableDefinition } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { Button, ColorPicker, Input, InputNumber, Segmented, Select, Slider, Space, Switch, Tooltip } from "antd";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";

import { SettingsOption } from "../../../../components/SettingsBox/index.tsx";
import { FontSelect } from "../../../../components/FontSelect/index.tsx";
import { CSS_UNITS } from "../constants.ts";
import { useThemeVariable } from "../hooks/useThemeVariable.ts";

export interface ThemeSettingProps {
  themeId: ThemeId;
  definition: ThemeVariableDefinition;
}

interface BooleanPair {
  trueVal: string;
  falseVal: string;
}

function detectBooleanOptions(options: unknown[]): BooleanPair | null {
  if (!options || options.length !== 2) {
    return null;
  }
  const first = options[0];
  const second = options[1];
  if (first == null || second == null) {
    return null;
  }
  const optA = String(first);
  const optB = String(second);
  const truthySet = new Set(["1", "1.0", "true", "on", "yes", "enabled", "enable"]);
  const falsySet = new Set(["0", "0.0", "false", "off", "no", "disabled", "disable"]);

  const aIsTruthy = truthySet.has(optA.toLowerCase());
  const bIsFalsy = falsySet.has(optB.toLowerCase());
  if (aIsTruthy && bIsFalsy) {
    return { trueVal: optA, falseVal: optB };
  }

  const bIsTruthy = truthySet.has(optB.toLowerCase());
  const aIsFalsy = falsySet.has(optA.toLowerCase());
  if (bIsTruthy && aIsFalsy) {
    return { trueVal: optB, falseVal: optA };
  }

  return null;
}

function isTruthyMatch(val: string | number | boolean | undefined, trueVal: string): boolean {
  if (val == null) return false;
  const s = String(val).trim().toLowerCase();
  const t = trueVal.trim().toLowerCase();
  if (s === t) return true;
  const numVal = Number(s);
  const numTrue = Number(t);
  if (!isNaN(numVal) && !isNaN(numTrue)) {
    return numVal === numTrue;
  }
  return false;
}

function isSwitchSetting(definition: ThemeVariableDefinition): boolean {
  if (definition.options && detectBooleanOptions(definition.options)) {
    return true;
  }
  if (definition.syntax === "<boolean>") {
    return true;
  }
  if (
    definition.syntax === "<number>" &&
    definition.min === 0 &&
    definition.max === 1 &&
    (definition.step == null || definition.step === 1)
  ) {
    return true;
  }
  return false;
}

function isSegmentedSetting(definition: ThemeVariableDefinition): boolean {
  if (!definition.options || detectBooleanOptions(definition.options)) {
    return false;
  }
  const opts = definition.options.map(String);
  const totalChars = opts.reduce((sum, o) => sum + o.length, 0);
  return opts.length >= 2 && opts.length <= 6 && totalChars <= 50;
}

function getInitialValueString(definition: ThemeVariableDefinition): string {
  if (definition.syntax === "<boolean>") {
    return definition.initialValue ? "1" : "0";
  }
  if (definition.syntax === "<length-percentage>") {
    return `${definition.initialValue}${definition.initialValueUnit ?? ""}`;
  }
  return String(definition.initialValue ?? "");
}

export function ThemeSetting({ themeId, definition }: ThemeSettingProps) {
  const { value: userStoredValue, onChange, onReset } = useThemeVariable(themeId, definition.name);
  const { t } = useTranslation();

  const isSwitch = isSwitchSetting(definition);
  const isSegmented = isSegmentedSetting(definition);
  const isStandaloneSlider = definition.syntax === "<number>" && definition.step != null;

  const initialValStr = getInitialValueString(definition).trim().toLowerCase();
  const currentValStr = (userStoredValue ?? initialValStr).trim().toLowerCase();
  const isModified = userStoredValue !== undefined && (
    definition.syntax === "<boolean>"
      ? (isTruthyMatch(currentValStr, "1") || isTruthyMatch(currentValStr, "true")) !==
        (isTruthyMatch(initialValStr, "1") || isTruthyMatch(initialValStr, "true"))
      : currentValStr !== initialValStr
  );

  const input = renderInput(definition, userStoredValue, onChange, onReset);
  const isLooseAction = isSwitch || isSegmented || isStandaloneSlider;

  return (
    <SettingsOption
      label={
        <Space size={6} align="center">
          <ResourceText text={definition.label} />
          {isModified && (
            <Tooltip title={t("resources.modified", "Modified")}>
              <span
                style={{
                  display: "inline-block",
                  width: 6,
                  height: 6,
                  borderRadius: "50%",
                  backgroundColor: "var(--ant-color-primary, #1677ff)",
                  verticalAlign: "middle",
                }}
              />
            </Tooltip>
          )}
        </Space>
      }
      tip={definition.tip ? <ResourceText text={definition.tip} /> : undefined}
      description={definition.description ? <ResourceText text={definition.description} /> : undefined}
      action={isLooseAction
        ? (
          <Space align="center" size={8}>
            {input}
            <Tooltip title={isModified ? t("reset_to_default") : undefined}>
              <Button onClick={onReset} disabled={!isModified}>
                <Icon iconName="BiReset" />
              </Button>
            </Tooltip>
          </Space>
        )
        : (
          <Space.Compact>
            {input}
            <Tooltip title={isModified ? t("reset_to_default") : undefined}>
              <Button onClick={onReset} disabled={!isModified}>
                <Icon iconName="BiReset" />
              </Button>
            </Tooltip>
          </Space.Compact>
        )}
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
    const boolPair = detectBooleanOptions(definition.options);
    if (boolPair) {
      const isChecked = isTruthyMatch(userStoredValue ?? String(definition.initialValue), boolPair.trueVal);
      return (
        <Switch
          checked={isChecked}
          onChange={(checked) => onChange(checked ? boolPair.trueVal : boolPair.falseVal)}
        />
      );
    }

    const opts = definition.options.map(String);
    if (opts.length >= 2 && opts.length <= 5 && opts.every((o) => o.length <= 16)) {
      const currentValue = userStoredValue ?? String(definition.initialValue);
      return (
        <Segmented
          options={opts.map((opt) => ({ label: opt, value: opt }))}
          value={opts.includes(currentValue) ? currentValue : opts[0]}
          onChange={(val) => onChange(String(val))}
        />
      );
    }

    return (
      <Select
        options={definition.options.map((value) => ({ value: String(value) }))}
        value={userStoredValue}
        defaultValue={`${definition.initialValue}`}
        onChange={onChange}
      />
    );
  }

  switch (definition.syntax) {
    case "<boolean>": {
      const isChecked = userStoredValue != null
        ? isTruthyMatch(userStoredValue, "1") || isTruthyMatch(userStoredValue, "true")
        : Boolean(definition.initialValue);
      return (
        <Switch
          checked={isChecked}
          onChange={(checked) => onChange(checked ? "1" : "0")}
        />
      );
    }

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
            min={definition.min || undefined}
            max={definition.max || undefined}
            step={definition.step || undefined}
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
      const { min, max, step } = definition;

      if (min === 0 && max === 1 && (step == null || step === 1)) {
        const isChecked = isTruthyMatch(userStoredValue ?? String(definition.initialValue), "1");
        return (
          <Switch
            checked={isChecked}
            onChange={(checked) => onChange(checked ? "1" : "0")}
          />
        );
      }

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
      const { min, max } = definition;
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

    case "<family-name>": {
      const value = userStoredValue ?? definition.initialValue;
      return <FontSelect value={value} onChange={onChange} />;
    }

    default: {
      return null;
    }
  }
}
