import { SeelenWindowManagerWidgetId } from '@seelen-ui/lib';
import { PluginId } from '@seelen-ui/lib/types';
import { ConfigProvider, Select, Switch } from 'antd';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { BorderSettings } from '../../border/infra';

import { newSelectors } from '../../../shared/store/app/reducer';
import { RootSelectors } from '../../../shared/store/app/selectors';
import { WManagerSettingsActions } from '../app';
import { ResourceText } from 'src/apps/shared/components/ResourceText';

import { SettingsGroup, SettingsOption } from '../../../../components/SettingsBox';
import { GlobalPaddings } from './GlobalPaddings';
import { OthersConfigs } from './Others';

export function WindowManagerSettings() {
  const settings = useSelector(RootSelectors.windowManager);
  const defaultLayout = useSelector(newSelectors.windowManager.defaultLayout);
  const plugins = useSelector(RootSelectors.plugins);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  const onToggleEnable = (value: boolean) => {
    dispatch(WManagerSettingsActions.setEnabled(value));
  };

  const onSelectLayout = (value: PluginId) => {
    dispatch(WManagerSettingsActions.setDefaultLayout(value));
  };

  const layouts = plugins.filter((plugin) => plugin.target === SeelenWindowManagerWidgetId);
  const usingLayout = layouts.find((plugin) => plugin.id === defaultLayout);

  return (
    <>
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
