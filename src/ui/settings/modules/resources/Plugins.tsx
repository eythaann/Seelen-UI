import { SeelenCommand } from '@seelen-ui/lib';
import { Plugin } from '@seelen-ui/lib/types';
import { Icon } from '@shared/components/Icon';
import { ResourceText } from '@shared/components/ResourceText';
import { path } from '@tauri-apps/api';
import { invoke } from '@tauri-apps/api/core';
import { Button } from 'antd';
import React from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import cs from './infra.module.css';

import { RootSelectors } from '../shared/store/app/selectors';

import { SettingsGroup, SettingsOption } from '../../components/SettingsBox';
import { ResourceCard } from './ResourceCard';

export function PluginsView() {
  const widgets = useSelector(RootSelectors.widgets);
  const plugins = useSelector(RootSelectors.plugins);

  const { t } = useTranslation();

  function targetLabel(target: string) {
    const widget = widgets.find((w) => w.id === target);
    if (widget) {
      return <ResourceText text={widget.metadata.displayName} />;
    }
    return <span>{target}</span>;
  }

  const groupedByTarget = plugins.reduce((acc, plugin) => {
    acc[plugin.target] ??= {
      label: targetLabel(plugin.target),
      plugins: [],
    };
    acc[plugin.target]!.plugins.push(plugin);
    return acc;
  }, {} as Record<string, { label: React.ReactNode; plugins: Plugin[] }>);

  Object.values(groupedByTarget).forEach((group) => {
    group.plugins.sort((a, b) => a.id.localeCompare(b.id));
  });

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>{t('resources.open_folder')}</b>
          <Button
            type="default"
            onClick={async () => {
              const dataDir = await path.appDataDir();
              invoke(SeelenCommand.OpenFile, { path: await path.join(dataDir, 'plugins') });
            }}
          >
            <Icon iconName="PiFoldersDuotone" />
          </Button>
        </SettingsOption>
        <SettingsOption>
          <span>{t('resources.discover')}:</span>
          <Button href="https://seelen.io/resources/s?category=Plugin" target="_blank" type="link">
            https://seelen.io/resources/s?category=Plugin
          </Button>
        </SettingsOption>
      </SettingsGroup>

      <div className={cs.list}>
        {Object.values(groupedByTarget).map((group, idx) => (
          <React.Fragment key={idx}>
            <b>{group.label}</b>
            {group.plugins.map((plugin) => (
              <ResourceCard
                key={plugin.id}
                resource={plugin}
                kind="Plugin"
                actions={
                  plugin.metadata.bundled ? undefined : (
                    <Button size="small" type="text" danger>
                      <Icon iconName="IoTrash" />
                    </Button>
                  )
                }
              />
            ))}
          </React.Fragment>
        ))}
      </div>
    </>
  );
}
