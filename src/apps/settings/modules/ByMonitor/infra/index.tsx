import { Button, Modal, Switch } from 'antd';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';
import { MonitorConfiguration, Rect } from 'seelen-core';

import { WindowManagerSpacingSettings } from '../../WindowManager/main/infra/GlobalPaddings';

import { newSelectors, RootActions } from '../../shared/store/app/reducer';
import { Monitor } from 'src/apps/settings/components/monitor';
import { SettingsGroup, SettingsOption } from 'src/apps/settings/components/SettingsBox';

import cs from './index.module.css';

interface MonitorConfigProps {
  index: number;
  monitor: MonitorConfiguration;
  onChange: (monitor: MonitorConfiguration) => void;
  onDelete: () => void;
  onInsert: () => void;
}

interface MoreMonitorConfigProps {
  index: number;
  monitor: MonitorConfiguration;
  onChange: (monitor: MonitorConfiguration) => void;
}

export function MoreMonitorConfig({ index, monitor: m, onChange }: MoreMonitorConfigProps) {
  const [open, setOpen] = useState(false);
  const { t } = useTranslation();

  function onChangeGap(v: number | null) {
    onChange({
      ...m,
      wm: {
        ...m.wm,
        gap: v ? Math.round(v) : null,
      },
    });
  }

  function onChangePadding(v: number | null) {
    onChange({
      ...m,
      wm: {
        ...m.wm,
        padding: v ? Math.round(v) : null,
      },
    });
  }

  function onChangeMargins(side: keyof Rect, v: number | null) {
    onChange({
      ...m,
      wm: {
        ...m.wm,
        margin: {
          ...(m.wm.margin || new Rect()),
          [side]: Math.round(v || 0),
        },
      },
    });
  }

  function onClear() {
    onChange({
      ...m,
      wm: {
        ...m.wm,
        gap: null,
        padding: null,
        margin: null,
      },
    });
  }

  return (
    <>
      <Button type="default" onClick={() => setOpen(true)}>
        {t('more')}
      </Button>
      <Modal
        title={t('monitors_configurations.label').replace('{{index}}', `${index + 1}`)}
        open={open}
        onCancel={() => setOpen(false)}
        centered
        footer={null}
      >
        <WindowManagerSpacingSettings
          gap={m.wm.gap}
          padding={m.wm.padding}
          margins={m.wm.margin}
          onChangeGap={onChangeGap}
          onChangePadding={onChangePadding}
          onChangeMargins={onChangeMargins}
          onClear={onClear}
        />
      </Modal>
    </>
  );
}

export function MonitorConfig({
  monitor: m,
  index,
  onChange,
  onDelete,
  onInsert,
}: MonitorConfigProps) {
  const { t } = useTranslation();

  function onToggle(key: string, value: boolean) {
    onChange({
      ...m,
      [key]: {
        enabled: value,
      },
    });
  }

  return (
    <SettingsGroup>
      <div className={cs.itemContainer}>
        <div className={cs.itemLeft}>
          <Monitor />
          <div className={cs.actions}>
            <Button type="primary" danger onClick={onDelete} disabled={index === 0}>
              {t('delete')}
            </Button>
            <Button type="primary" onClick={onInsert}>
              {t('insert')}
            </Button>
            <MoreMonitorConfig index={index} monitor={m} onChange={onChange} />
          </div>
        </div>
        <SettingsGroup>
          <SettingsOption>
            <b>{t('toolbar.enable')}</b>
            <Switch value={m.tb.enabled} onChange={(v) => onToggle('tb', v)} />
          </SettingsOption>
          <SettingsOption>
            <b>{t('wm.enable')}</b>
            <Switch value={m.wm.enabled} onChange={(v) => onToggle('wm', v)} />
          </SettingsOption>
          <SettingsOption>
            <b>{t('weg.enable')}</b>
            <Switch value={m.weg.enabled} onChange={(v) => onToggle('weg', v)} />
          </SettingsOption>
          <SettingsOption>
            <b>{t('wall.enable')}</b>
            <Switch value={m.wall.enabled} onChange={(v) => onToggle('wall', v)} />
          </SettingsOption>
        </SettingsGroup>
      </div>
    </SettingsGroup>
  );
}

export function SettingsByMonitor() {
  const monitors = useSelector(newSelectors.monitors);

  const dispatch = useDispatch();

  function onMonitorChange(idx: number, monitor: MonitorConfiguration) {
    let newMonitors = [...monitors];
    newMonitors[idx] = monitor;
    dispatch(RootActions.setMonitors(newMonitors));
  }

  function insertNewAfter(idx: number) {
    let newMonitors = [...monitors];
    newMonitors.splice(idx + 1, 0, new MonitorConfiguration());
    dispatch(RootActions.setMonitors(newMonitors));
  }

  function onDelete(idx: number) {
    let newMonitors = [...monitors];
    newMonitors.splice(idx, 1);
    dispatch(RootActions.setMonitors(newMonitors));
  }

  return (
    <>
      {monitors.map((m, idx) => (
        <MonitorConfig
          key={idx}
          index={idx}
          monitor={m}
          onChange={onMonitorChange.bind(null, idx)}
          onDelete={onDelete.bind(null, idx)}
          onInsert={insertNewAfter.bind(null, idx)}
        />
      ))}
    </>
  );
}
