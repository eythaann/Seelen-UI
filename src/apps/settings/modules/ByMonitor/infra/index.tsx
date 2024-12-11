import { Button, Modal, Switch } from 'antd';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';
import { ConnectedMonitor, MonitorConfiguration, Rect } from 'seelen-core';

import { WindowManagerSpacingSettings } from '../../WindowManager/main/infra/GlobalPaddings';

import { newSelectors, RootActions } from '../../shared/store/app/reducer';
import { Monitor } from 'src/apps/settings/components/monitor';
import { SettingsGroup, SettingsOption } from 'src/apps/settings/components/SettingsBox';

import cs from './index.module.css';

interface MonitorConfigProps {
  device: ConnectedMonitor;
  config: MonitorConfiguration;
  onChange: (monitor: MonitorConfiguration) => void;
}

export function MoreMonitorConfig({ device, config, onChange }: MonitorConfigProps) {
  const [open, setOpen] = useState(false);
  const { t } = useTranslation();

  function onChangeGap(v: number | null) {
    onChange({
      ...config,
      wm: {
        ...config.wm,
        gap: v ? Math.round(v) : null,
      },
    });
  }

  function onChangePadding(v: number | null) {
    onChange({
      ...config,
      wm: {
        ...config.wm,
        padding: v ? Math.round(v) : null,
      },
    });
  }

  function onChangeMargins(side: keyof Rect, v: number | null) {
    onChange({
      ...config,
      wm: {
        ...config.wm,
        margin: {
          ...(config.wm.margin || new Rect()),
          [side]: Math.round(v || 0),
        },
      },
    });
  }

  function onClear() {
    onChange({
      ...config,
      wm: {
        ...config.wm,
        gap: null,
        padding: null,
        margin: null,
      },
    });
  }

  // todo(eythan) more settings will be disabled until we have a finished implementation
  return (
    <>
      <Button type="default" onClick={() => setOpen(true)} disabled>
        {t('more')}
      </Button>
      <Modal
        title={t('monitors_configurations.label').replace('{{index}}', device.name)}
        open={open}
        onCancel={() => setOpen(false)}
        centered
        footer={null}
      >
        <WindowManagerSpacingSettings
          gap={config.wm.gap}
          padding={config.wm.padding}
          margins={config.wm.margin}
          onChangeGap={onChangeGap}
          onChangePadding={onChangePadding}
          onChangeMargins={onChangeMargins}
          onClear={onClear}
        />
      </Modal>
    </>
  );
}

export function MonitorConfig({ device, config, onChange }: MonitorConfigProps) {
  const { t } = useTranslation();

  function onToggle(key: string, value: boolean) {
    onChange({
      ...config,
      [key]: {
        enabled: value,
      },
    });
  }

  return (
    <SettingsGroup>
      <div className={cs.itemContainer}>
        <div className={cs.itemLeft}>
          <div className={cs.label}>{device.name}</div>
          <Monitor width={device.width} height={device.height} />
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
              <Switch value={config.tb.enabled} onChange={(v) => onToggle('tb', v)} />
            </SettingsOption>
            <SettingsOption>
              <b>{t('wm.enable')}</b>
              <Switch value={config.wm.enabled} onChange={(v) => onToggle('wm', v)} disabled />
            </SettingsOption>
            <SettingsOption>
              <b>{t('weg.enable')}</b>
              <Switch value={config.weg.enabled} onChange={(v) => onToggle('weg', v)} />
            </SettingsOption>
            <SettingsOption>
              <b>{t('wall.enable')}</b>
              <Switch value={config.wall.enabled} onChange={(v) => onToggle('wall', v)} disabled />
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

  function onMonitorChange(id: string, monitor: MonitorConfiguration) {
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
        let monitor = settingsByMonitor[device.id] || new MonitorConfiguration();
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
