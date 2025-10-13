import type { ThemeConfigDefinition, ThemeId, ThemeVariableDefinition } from "@seelen-ui/lib/types";
import { Icon } from "@shared/components/Icon";
import { ResourceText } from "@shared/components/ResourceText";
import { Button, ColorPicker, Input, InputNumber, Select, Space, Tooltip } from "antd";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { useDispatch, useSelector } from "react-redux";
import { useParams } from "react-router";

import { RootActions } from "../../shared/store/app/reducer.ts";

import type { RootState } from "../../shared/store/domain.ts";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../../components/SettingsBox/index.tsx";

export function ThemeView() {
  const { username, resourceName } = useParams<"username" | "resourceName">();
  const theme = useSelector((state: RootState) => {
    return state.availableThemes.find((t) => t.id === `@${username}/${resourceName}`);
  });

  const { t } = useTranslation();
  const d = useDispatch();

  function onReset() {
    d(RootActions.resetThemeVariables({ themeId: theme!.id }));
  }

  if (!theme) {
    return <div>wow 404 !?</div>;
  }

  return (
    <>
      <SettingsGroup>
        <SettingsOption
          label={t("reset_all_to_default")}
          action={
            <Button onClick={onReset}>
              <Icon iconName="RiResetLeftLine" />
            </Button>
          }
        />
      </SettingsGroup>
      {theme.settings.map((def, idx) => <ThemeConfigDefinition key={idx} themeId={theme.id} def={def} />)}
    </>
  );
}

interface ThemeConfigDefinitionProps {
  def: ThemeConfigDefinition;
  themeId: ThemeId;
  nestLevel?: number;
}

function ThemeConfigDefinition(
  { def, themeId, nestLevel = 0 }: ThemeConfigDefinitionProps,
) {
  let subGroupElement: ReactNode = null;

  if ("group" in def) {
    subGroupElement = (
      <SettingsSubGroup label={<ResourceText text={def.group.header} />}>
        {def.group.items.map((item, idx) => (
          <ThemeConfigDefinition
            key={idx}
            themeId={themeId}
            def={item}
            nestLevel={nestLevel + 1}
          />
        ))}
      </SettingsSubGroup>
    );
  } else {
    subGroupElement = <ThemeSetting themeId={themeId} definition={def} />;
  }

  return nestLevel === 0 ? <SettingsGroup>{subGroupElement}</SettingsGroup> : subGroupElement;
}

interface ThemeSettingProps {
  themeId: ThemeId;
  definition: ThemeVariableDefinition;
}

function ThemeSetting({ themeId, definition }: ThemeSettingProps) {
  const userStoredValue = useSelector(
    (state: RootState) => state.byTheme[themeId]?.[definition.name],
  );

  const { t } = useTranslation();
  const d = useDispatch();

  const onChangeVarValue = (value: string) => {
    d(RootActions.setThemeVariable({ themeId, name: definition.name, value }));
  };

  const onDeleteVarValue = () => {
    d(RootActions.deleteThemeVariable({ themeId, name: definition.name }));
  };

  let optionInput: ReactNode = null;
  switch (definition.syntax) {
    case "<color>": {
      optionInput = (
        <ColorPicker
          showText
          onChangeComplete={(v) => {
            onChangeVarValue(v.toHexString());
          }}
          value={userStoredValue || definition.initialValue}
        />
      );
      break;
    }
    case "<length>": {
      const value = userStoredValue ? parseFloat(userStoredValue) : definition.initialValue;
      const unit = userStoredValue?.replace(/[\d\.]+/, "") ||
        definition.initialValueUnit;
      optionInput = (
        <Space.Compact>
          <InputNumber
            value={value}
            onChange={(newValue) => {
              if (newValue == null) {
                onDeleteVarValue();
                return;
              }
              onChangeVarValue(`${newValue}${unit}`);
            }}
          />
          <Select
            options={CSS_UNITS.map((value) => ({ value }))}
            style={{ width: 60, minWidth: 60 }}
            value={unit}
            onChange={(unit) => {
              onChangeVarValue(`${value}${unit}`);
            }}
          />
        </Space.Compact>
      );
      break;
    }
    case "<number>": {
      const value = userStoredValue ? parseFloat(userStoredValue) : definition.initialValue;
      optionInput = (
        <InputNumber
          value={value}
          onChange={(newValue) => {
            if (newValue == null) {
              onDeleteVarValue();
              return;
            }
            onChangeVarValue(`${newValue}`);
          }}
        />
      );
      break;
    }
    case "<url>": {
      optionInput = (
        <Input
          value={userStoredValue ?? definition.initialValue}
          onChange={(e) => {
            onChangeVarValue(e.currentTarget.value);
          }}
        />
      );
      break;
    }
    case "<string>": {
      optionInput = (
        <Input
          value={userStoredValue ?? definition.initialValue}
          onChange={(e) => {
            onChangeVarValue(e.currentTarget.value);
          }}
        />
      );
      break;
    }
    default: {
      // @ts-expect-error should never happen
      definition.syntax;
    }
  }

  return (
    <SettingsOption
      label={<ResourceText text={definition.label} />}
      tip={definition.tip ? <ResourceText text={definition.tip} /> : undefined}
      description={definition.description ? <ResourceText text={definition.description} /> : undefined}
      action={
        <Space.Compact>
          {optionInput}
          <Tooltip title={t("reset_to_default")}>
            <Button onClick={onDeleteVarValue}>
              <Icon iconName="BiReset" />
            </Button>
          </Tooltip>
        </Space.Compact>
      }
    />
  );
}

const CSS_UNITS = ["px", "%", "rem", "em", "vh", "vw"];
