import { SeelenWindowManagerWidgetId, VirtualDesktopStrategy } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';
import { Alert, Button, ConfigProvider, Select, Switch } from 'antd';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { BorderSettings } from '../../border/infra';

import { newSelectors, RootActions } from '../../../shared/store/app/reducer';
import { RootSelectors } from '../../../shared/store/app/selectors';
import { WManagerSettingsActions } from '../app';
import { ResourceText } from 'src/apps/shared/components/ResourceText';

import { SettingsGroup, SettingsOption } from '../../../../components/SettingsBox';
import { GlobalPaddings } from './GlobalPaddings';
import { OthersConfigs } from './Others';

export function WindowManagerSettings() {
  const [isWinVerSupported, setIsWinVerSupported] = useState(false);

  const vdStrategy = useSelector(RootSelectors.virtualDesktopStrategy);
  const settings = useSelector(RootSelectors.windowManager);
  const defaultLayout = useSelector(newSelectors.windowManager.defaultLayout);
  const plugins = useSelector(RootSelectors.plugins);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  useEffect(() => {
    invoke<boolean>('is_virtual_desktop_supported').then(setIsWinVerSupported);
  }, []);

  const onSelectVirtualDesktopStrategy = (value: VirtualDesktopStrategy) => {
    if (value !== vdStrategy) {
      dispatch(RootActions.setVirtualDesktopStrategy(value));
    }
  };

  const onToggleEnable = (value: boolean) => {
    dispatch(WManagerSettingsActions.setEnabled(value));
  };

  const onSelectLayout = (value: string) => {
    dispatch(WManagerSettingsActions.setDefaultLayout(value));
  };

  const layouts = plugins.filter((plugin) => plugin.target === SeelenWindowManagerWidgetId);
  const usingLayout = layouts.find((plugin) => plugin.id === defaultLayout);

  return (
    <>
      {!isWinVerSupported && (
        <Alert
          type="info"
          showIcon
          message={t('vd.disabled_windows_version')}
          style={{ marginBottom: 10 }}
        />
      )}

      <SettingsGroup>
        <SettingsOption>
          <div>
            <b>{t('vd.strategy.label')}</b>
          </div>
          <Button.Group>
            <Button
              disabled={!isWinVerSupported}
              type={vdStrategy === VirtualDesktopStrategy.Native ? 'primary' : 'default'}
              onClick={() => onSelectVirtualDesktopStrategy(VirtualDesktopStrategy.Native)}
            >
              {t('vd.strategy.native')}
            </Button>
            <Button
              type={vdStrategy === VirtualDesktopStrategy.Seelen ? 'primary' : 'default'}
              onClick={() => onSelectVirtualDesktopStrategy(VirtualDesktopStrategy.Seelen)}
            >
              {t('vd.strategy.seelen')}
            </Button>
          </Button.Group>
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <div>
            <b>{t('wm.enable')}</b>
          </div>
          <Switch checked={settings.enabled} onChange={onToggleEnable} />
        </SettingsOption>
      </SettingsGroup>

      <ConfigProvider componentDisabled={!settings.enabled}>
        <SettingsGroup>
          <SettingsOption>
            <div>
              <b>{t('wm.layout')}: </b>
            </div>
            <Select
              style={{ width: '200px' }}
              value={defaultLayout}
              options={layouts.map((layout) => ({
                key: layout.id,
                label: <ResourceText text={layout.metadata.displayName} />,
                value: layout.id,
              }))}
              onSelect={onSelectLayout}
            />
          </SettingsOption>
          <div>
            <p>
              <b>{t('wm.author')}: </b>
              <ResourceText text={usingLayout?.metadata.author || '-'} />,
            </p>
            <p>
              <b>{t('wm.description')}: </b>
              <ResourceText text={usingLayout?.metadata.description || '-'} />,
            </p>
          </div>
        </SettingsGroup>

        <GlobalPaddings />
        <OthersConfigs />
        <SettingsGroup>
          <BorderSettings />
        </SettingsGroup>
      </ConfigProvider>
    </>
  );
}
