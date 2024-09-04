import { VariableConvention } from '../../../shared/schemas';
import { SettingsGroup, SettingsOption } from '../../components/SettingsBox';
import { Input, Switch, Tooltip } from 'antd';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';
import { AhkVarList } from 'seelen-core';

import { RootActions } from '../shared/store/app/reducer';
import { RootSelectors } from '../shared/store/app/selectors';
import { AhkVariablesActions, KeyCodeToAHK } from './app';

export function Shortcuts() {
  const ahkEnable = useSelector(RootSelectors.ahkEnabled);
  const ahkVariables = useSelector(RootSelectors.ahkVariables);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  const onChangeEnabled = (value: boolean) => {
    dispatch(RootActions.setAhkEnabled(value));
    dispatch(RootActions.setToBeSaved(true));
  };

  const onChangeVar = (name: keyof AhkVarList, e: React.KeyboardEvent<HTMLInputElement>) => {
    const result = KeyCodeToAHK(e);
    if (result) {
      dispatch(AhkVariablesActions.setVariable({ name, value: result }));
    }
  };

  return (
    <div>
      <SettingsGroup>
        <SettingsOption>
          <span>
            {t('shortcuts.enable')}{' '}
            <Tooltip
              title={t('shortcuts.enable_tooltip')}
            >
              🛈
            </Tooltip>
          </span>
          <Switch value={ahkEnable} onChange={onChangeEnabled} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        {
          Object.entries(ahkVariables).map(([key, value]) => {
            return (
              <SettingsOption key={key}>
                <div>{t(`shortcuts.labels.${VariableConvention.camelToSnake(key)}`)}</div>
                <Input
                  value={value.fancy}
                  onKeyDown={(e) => onChangeVar(key as keyof AhkVarList, e)}
                />
              </SettingsOption>
            );
          })
        }
      </SettingsGroup>
    </div>
  );
}
