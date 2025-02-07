import { AhkVarList } from '@seelen-ui/lib/types';
import { Button, Input, Switch, Tooltip } from 'antd';
import { pick } from 'lodash';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { RootActions } from '../shared/store/app/reducer';
import { RootSelectors } from '../shared/store/app/selectors';
import { AhkVariablesActions, KeyCodeToAHK } from './app';
import { Icon } from 'src/apps/shared/components/Icon';

import { VariableConvention } from '../../../shared/schemas';
import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../components/SettingsBox';

interface AhkOptionsProps {
  variables: Array<keyof AhkVarList>;
  onChangeVar: anyFunction;
}

function AhkOptions({ variables, onChangeVar }: AhkOptionsProps) {
  const all = useSelector(RootSelectors.ahkVariables);

  const { t } = useTranslation();

  const toUse = pick(all, variables);

  return Object.entries(toUse).map(([key, value]) => {
    return (
      <SettingsOption key={key}>
        <div>{t(`shortcuts.labels.${VariableConvention.camelToSnake(key)}`)}</div>
        <Tooltip title={value.readonly ? t('shortcuts.readonly_tooltip') : null}>
          <Input
            value={value.fancy}
            disabled={value.readonly}
            onKeyDown={(e) => {
              if (!value.readonly) {
                onChangeVar(key as keyof AhkVarList, e);
              }
            }}
          />
        </Tooltip>
      </SettingsOption>
    );
  });
}

export function Shortcuts() {
  const ahkEnable = useSelector(RootSelectors.ahkEnabled);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  function onChangeEnabled(value: boolean) {
    dispatch(RootActions.setAhkEnabled(value));
    dispatch(RootActions.setToBeSaved(true));
  }

  function onChangeVar(name: keyof AhkVarList, e: React.KeyboardEvent<HTMLInputElement>) {
    const result = KeyCodeToAHK(e);
    if (result) {
      dispatch(AhkVariablesActions.setVariable({ name, value: result }));
    }
  }

  function onReset() {
    dispatch(AhkVariablesActions.reset());
  }

  return (
    <div>
      <SettingsGroup>
        <SettingsOption>
          <b style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
            {t('shortcuts.enable')}
            <Tooltip title={t('shortcuts.enable_tooltip')}>
              <Icon iconName="TbHelpHexagon" />
            </Tooltip>
          </b>
          <Switch value={ahkEnable} onChange={onChangeEnabled} />
        </SettingsOption>
        <SettingsOption>
          <span>{t('shortcuts.reset')}</span>
          <Button onClick={onReset}>
            <Icon iconName="TbRefresh" />
          </Button>
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t('header.labels.seelen_rofi')}>
          <AhkOptions variables={['toggleLauncher']} onChangeVar={onChangeVar} />
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t('header.labels.seelen_wm')}>
          <AhkOptions
            variables={['focusLeft', 'focusRight', 'focusTop', 'focusBottom']}
            onChangeVar={onChangeVar}
          />
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t('header.labels.seelen_wm')}>
          <AhkOptions
            variables={[
              'increaseWidth',
              'decreaseWidth',
              'increaseHeight',
              'decreaseHeight',
              'restoreSizes',
            ]}
            onChangeVar={onChangeVar}
          />
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t('header.labels.seelen_wm')}>
          <AhkOptions
            variables={[
              'reserveLeft',
              'reserveRight',
              'reserveTop',
              'reserveBottom',
              'reserveStack',
              'reserveFloat',
            ]}
            onChangeVar={onChangeVar}
          />
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t('header.labels.virtual_desk')}>
          <AhkOptions
            variables={[
              'switchWorkspace0',
              'switchWorkspace1',
              'switchWorkspace2',
              'switchWorkspace3',
              'switchWorkspace4',
              'switchWorkspace5',
              'switchWorkspace6',
              'switchWorkspace7',
              'switchWorkspace8',
              'switchWorkspace9',
            ]}
            onChangeVar={onChangeVar}
          />
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t('header.labels.virtual_desk')}>
          <AhkOptions
            variables={[
              'sendToWorkspace0',
              'sendToWorkspace1',
              'sendToWorkspace2',
              'sendToWorkspace3',
              'sendToWorkspace4',
              'sendToWorkspace5',
              'sendToWorkspace6',
              'sendToWorkspace7',
              'sendToWorkspace8',
              'sendToWorkspace9',
            ]}
            onChangeVar={onChangeVar}
          />
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t('header.labels.virtual_desk')}>
          <AhkOptions
            variables={[
              'moveToWorkspace0',
              'moveToWorkspace1',
              'moveToWorkspace2',
              'moveToWorkspace3',
              'moveToWorkspace4',
              'moveToWorkspace5',
              'moveToWorkspace6',
              'moveToWorkspace7',
              'moveToWorkspace8',
              'moveToWorkspace9',
            ]}
            onChangeVar={onChangeVar}
          />
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t('header.labels.seelen_weg')}>
          <AhkOptions
            variables={[
              'startWegApp0',
              'startWegApp1',
              'startWegApp2',
              'startWegApp3',
              'startWegApp4',
              'startWegApp5',
              'startWegApp6',
              'startWegApp7',
              'startWegApp8',
              'startWegApp9',
            ]}
            onChangeVar={onChangeVar}
          />
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t('miscellaneous')}>
          <AhkOptions
            variables={['miscOpenSettings', 'miscToggleLockTracing', 'miscToggleWinEventTracing']}
            onChangeVar={onChangeVar}
          />
        </SettingsSubGroup>
      </SettingsGroup>
    </div>
  );
}
