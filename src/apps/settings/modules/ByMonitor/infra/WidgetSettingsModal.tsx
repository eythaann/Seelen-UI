// This file is for testing, not final implementation yet.

import { MonitorConfiguration, MonitorSettingsByWidget } from '@seelen-ui/lib/types';
import { Button, Modal, Select, Switch } from 'antd';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { newSelectors, RootActions } from '../../shared/store/app/reducer';
import {
  SettingsGroup,
  SettingsOption,
  SettingsSubGroup,
} from 'src/apps/settings/components/SettingsBox';
import { Icon } from 'src/apps/shared/components/Icon';

import { WidgetConfigDeclarationItem, WidgetSettingsDeclarationGroup } from '../domain';

interface Props {
  widgetId: string;
  monitorId: string;
  settings: WidgetSettingsDeclarationGroup[];
}

export function WidgetSettingsModal({ widgetId, monitorId, settings: declarations }: Props) {
  const [open, setOpen] = useState(false);

  const settingsByMonitor = useSelector(newSelectors.monitorsV2);
  const monitorConfig = settingsByMonitor[monitorId];
  const widgetConfig = monitorConfig?.byWidget[widgetId];

  const dispatch = useDispatch();

  if (!monitorConfig || !widgetConfig) {
    return null;
  }

  const patchMonitor = (monitor: Partial<MonitorConfiguration>) => {
    dispatch(
      RootActions.setMonitors({
        ...settingsByMonitor,
        [monitorId]: {
          ...monitorConfig,
          ...monitor,
        },
      }),
    );
  };

  const patchByWidgetSettings = (patch: Partial<MonitorSettingsByWidget>) => {
    patchMonitor({
      byWidget: {
        ...monitorConfig.byWidget,
        ...patch,
      },
    });
  };

  const patchWidgetSettings = (patch: Record<string, any>) => {
    patchByWidgetSettings({
      [widgetId]: {
        ...widgetConfig,
        ...patch,
      },
    });
  };

  const onChangeItem = (key: string, value: any) => {
    patchWidgetSettings({ [key]: value });
  };

  return (
    <>
      <Modal open={open} onCancel={() => setOpen(false)} title="-" footer={null} centered>
        {declarations.map((group, index) => (
          <SettingsGroup key={index}>
            {group.settings.map((item, index) => {
              if ('settings' in item) {
                return (
                  <SettingsSubGroup key={index} label={item.title}>
                    {item.settings.map((item) => (
                      <ItemByDeclaration
                        key={item.key}
                        value={widgetConfig[item.key]}
                        declaration={item}
                        onChange={onChangeItem.bind(null, item.key)}
                      />
                    ))}
                  </SettingsSubGroup>
                );
              }

              return (
                <ItemByDeclaration
                  key={item.key}
                  value={widgetConfig[item.key]}
                  declaration={item}
                  onChange={onChangeItem.bind(null, item.key)}
                />
              );
            })}
          </SettingsGroup>
        ))}
      </Modal>
      <Button type="default" onClick={() => setOpen(true)}>
        <Icon iconName="RiSettings4Fill" />
      </Button>
    </>
  );
}

interface ItemProps {
  value: any;
  onChange(value: any): void;
  declaration: WidgetConfigDeclarationItem;
}

function ItemByDeclaration({ value, declaration, onChange }: ItemProps) {
  const { t } = useTranslation();

  const translateIfNeeded = (v: string): string => {
    if (v.startsWith('t::')) {
      return t(v.slice(3));
    }
    return v;
  };

  let configElement = null;

  if (declaration.type === 'switch') {
    configElement = <Switch value={value} defaultValue={declaration.default} onChange={onChange} />;
  }

  if (declaration.type === 'select') {
    configElement = (
      <Select
        style={{ width: '120px' }}
        value={value}
        defaultValue={declaration.default}
        placeholder={t('inherit')}
        allowClear
        options={declaration.options.map((option) => ({
          ...option,
          label: translateIfNeeded(option.label),
        }))}
        onChange={onChange}
      />
    );
  }

  return (
    <SettingsOption>
      <span>{translateIfNeeded(declaration.label)}</span>
      {configElement}
    </SettingsOption>
  );
}
