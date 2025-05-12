import {
  SeelenToolbarWidgetId,
  SeelenWallWidgetId,
  SeelenWegWidgetId,
  SeelenWindowManagerWidgetId,
} from '@seelen-ui/lib';
import {
  MonitorConfiguration as IMonitorConfiguration,
  PhysicalMonitor,
  WidgetId,
} from '@seelen-ui/lib/types';
import { Switch } from 'antd';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { newSelectors, RootActions } from '../../shared/store/app/reducer';
import { WegSettingsDeclaration } from '../application';
import { Monitor } from 'src/apps/settings/components/monitor';
import { SettingsGroup, SettingsOption } from 'src/apps/settings/components/SettingsBox';

import { WidgetSettingsModal } from './WidgetSettingsModal';
import cs from './index.module.css';

interface MonitorConfigProps {
  device: PhysicalMonitor;
  config: IMonitorConfiguration;
  onChange: (monitor: IMonitorConfiguration) => void;
}

export function MonitorConfig({ device, config, onChange }: MonitorConfigProps) {
  const { t } = useTranslation();

  function onToggle(key: WidgetId, value: boolean) {
    onChange({
      ...config,
      byWidget: {
        ...config.byWidget,
        [key]: {
          enabled: value,
        },
      },
    });
  }

  return (
    <SettingsGroup>
      <div className={cs.itemContainer}>
        <div className={cs.itemLeft}>
          <div className={cs.label}>{device.name}</div>
          <Monitor
            width={device.rect.right - device.rect.left}
            height={device.rect.bottom - device.rect.top}
          />
          {/* <div className={cs.actions}>
            <Button type="primary" danger>
              {t('delete')}
            </Button>
            <Button type="primary">{t('insert')}</Button>
            <MoreMonitorConfig device={device} config={config} onChange={onChange} />
          </div> */}
        </div>
        <div className={cs.itemRight}>
          <SettingsGroup>
            <SettingsOption>
              <b>{t('toolbar.enable')}</b>
              <Switch
                value={!!config.byWidget[SeelenToolbarWidgetId]?.enabled}
                onChange={(v) => onToggle(SeelenToolbarWidgetId, v)}
              />
            </SettingsOption>
            <SettingsOption>
              <b>{t('wm.enable')}</b>
              <Switch
                value={!!config.byWidget[SeelenWindowManagerWidgetId]?.enabled}
                onChange={(v) => onToggle(SeelenWindowManagerWidgetId, v)}
                disabled
              />
            </SettingsOption>
            <SettingsOption>
              <b>{t('header.labels.seelen_weg')}</b>
              <WidgetSettingsModal
                widgetId={SeelenWegWidgetId}
                monitorId={device.id}
                settings={WegSettingsDeclaration}
              />
            </SettingsOption>
            <SettingsOption>
              <b>{t('wall.enable')}</b>
              <Switch
                value={!!config.byWidget[SeelenWallWidgetId]?.enabled}
                onChange={(v) => onToggle(SeelenWallWidgetId, v)}
                disabled
              />
            </SettingsOption>
          </SettingsGroup>
        </div>
      </div>
    </SettingsGroup>
  );
}

export function SettingsByMonitor() {
  const devices = useSelector(newSelectors.connectedMonitors);
  const settingsByMonitor = useSelector(newSelectors.monitorsV2);

  const dispatch = useDispatch();

  function onMonitorChange(id: string, monitor: IMonitorConfiguration) {
    dispatch(
      RootActions.setMonitors({
        ...settingsByMonitor,
        [id]: monitor,
      }),
    );
  }

  return (
    <>
      {devices.map((device) => {
        let monitor = settingsByMonitor[device.id];
        if (!monitor) {
          return null;
        }
        return (
          <MonitorConfig
            key={device.id}
            device={device}
            config={monitor}
            onChange={onMonitorChange.bind(null, device.id)}
          />
        );
      })}
    </>
  );
}
