import { ThemeId, ThemeVariableDefinition } from '@seelen-ui/lib/types';
import { ResourceText } from '@shared/components/ResourceText';
import { ColorPicker, Input, InputNumber, Select, Space } from 'antd';
import { ReactNode } from 'react';
import { useDispatch, useSelector } from 'react-redux';
import { useParams } from 'react-router';

import { RootActions } from '../../shared/store/app/reducer';
import {
  SettingsGroup,
  SettingsOption,
  SettingsSubGroup,
} from 'src/apps/settings/components/SettingsBox';

import { RootState } from '../../shared/store/domain';

export function ThemeView() {
  const { username, resourceName } = useParams<'username' | 'resourceName'>();
  const theme = useSelector((state: RootState) => {
    return state.availableThemes.find((t) => t.id === `@${username}/${resourceName}`);
  });

  if (!theme) {
    return <div>wow 404 !?</div>;
  }

  return (
    <SettingsGroup>
      <SettingsSubGroup label={<b>{theme.id}</b>}>
        {theme.settings.map((setting) => (
          <ThemeSetting key={setting.name} themeId={theme.id} definition={setting} />
        ))}
      </SettingsSubGroup>
    </SettingsGroup>
  );
}

interface ThemeSettingProps {
  themeId: ThemeId;
  definition: ThemeVariableDefinition;
}

function ThemeSetting({ themeId, definition }: ThemeSettingProps) {
  const variableValue = useSelector(
    (state: RootState) => state.byTheme[themeId]?.[definition.name],
  );

  const d = useDispatch();

  const onChangeVarValue = (value: string) => {
    d(RootActions.setThemeVariable({ themeId, name: definition.name, value }));
  };

  const onDeleteVarValue = () => {
    d(RootActions.deleteThemeVariable({ themeId, name: definition.name }));
  };

  let optionInput: ReactNode = null;
  switch (definition.syntax) {
    case '<color>': {
      optionInput = (
        <ColorPicker
          showText
          onChangeComplete={(v) => {
            onChangeVarValue(v.toHexString());
          }}
          value={variableValue || definition.initialValue}
        />
      );
      break;
    }
    case '<length>': {
      const value = variableValue ? parseFloat(variableValue) : definition.initialValue;
      const unit = variableValue?.replace(/[\d\.]+/, '') || definition.initialValueUnit;
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
    case '<number>': {
      const value = variableValue ? parseFloat(variableValue) : definition.initialValue;
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
    case '<url>': {
      optionInput = (
        <Input
          value={variableValue}
          defaultValue={definition.initialValue}
          onChange={(e) => {
            onChangeVarValue(e.currentTarget.value);
          }}
        />
      );
      break;
    }
    case '<string>': {
      optionInput = (
        <Input
          value={variableValue}
          defaultValue={definition.initialValue}
          onChange={(e) => {
            onChangeVarValue(e.currentTarget.value);
          }}
        />
      );
      break;
    }
    default: {
      // @ts-expect-error
      definition.syntax;
    }
  }

  return (
    <SettingsOption>
      <ResourceText text={definition.label} />
      {optionInput}
    </SettingsOption>
  );
}

const CSS_UNITS = ['px', '%', 'rem', 'em', 'vh', 'vw'];
