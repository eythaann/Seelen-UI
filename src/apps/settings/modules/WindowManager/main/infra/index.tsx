import { SettingsGroup, SettingsOption } from '../../../../components/SettingsBox';
import { GlobalPaddings } from './GlobalPaddings';
import { OthersConfigs } from './Others';
import { invoke } from '@tauri-apps/api/core';
import { Select, Switch } from 'antd';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { BorderSettings } from '../../border/infra';

import { newSelectors } from '../../../shared/store/app/reducer';
import { RootSelectors } from '../../../shared/store/app/selectors';
import { WManagerSettingsActions } from '../app';

export function WindowManagerSettings() {
  const [isWindows10, setIsWindows10] = useState(false);

  const settings = useSelector(RootSelectors.windowManager);
  const layouts = useSelector(newSelectors.availableLayouts);
  const defaultLayout = useSelector(newSelectors.windowManager.defaultLayout);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  useEffect(() => {
    invoke<string>('get_win_version').then((ver) => setIsWindows10(ver === 'Windows10'));
  }, []);

  const onToggleEnable = (value: boolean) => {
    dispatch(WManagerSettingsActions.setEnabled(value));
  };

  const onSelectLayout = (value: string) => {
    dispatch(WManagerSettingsActions.setDefaultLayout(value));
  };

  const usingLayout = layouts.find((layout) => layout.info.filename === defaultLayout);

  return (
    <>
      {isWindows10 && (
        <SettingsGroup>
          <div>
            <p>{t('wm.disabled_windows10')}</p>
          </div>
        </SettingsGroup>
      )}

      <SettingsGroup>
        <SettingsOption>
          <div>
            <b>{t('wm.enable')}</b>
          </div>
          <Switch checked={settings.enabled} onChange={onToggleEnable} disabled={isWindows10} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <div>
            <b>{t('wm.layout')}: </b>
          </div>
          <Select
            style={{ width: '200px' }}
            value={defaultLayout}
            options={layouts.map((layout, idx) => ({
              key: `layout-${idx}`,
              label: layout.info.displayName,
              value: layout.info.filename,
            }))}
            onSelect={onSelectLayout}
          />
        </SettingsOption>
        <div>
          <p>
            <b>{t('wm.author')}: </b>
            {usingLayout?.info.author}
          </p>
          <p>
            <b>{t('wm.description')}: </b>
            {usingLayout?.info.description}
          </p>
        </div>
      </SettingsGroup>

      <GlobalPaddings />
      <OthersConfigs />
      <SettingsGroup>
        <BorderSettings />
      </SettingsGroup>
    </>
  );
}
