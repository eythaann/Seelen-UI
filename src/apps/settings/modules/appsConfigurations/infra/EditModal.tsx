import { createSelector } from '@reduxjs/toolkit';
import { ConfigProvider, Input, Modal, Select, Switch } from 'antd';
import React, { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { ownSelector, RootSelectors } from '../../shared/store/app/selectors';

import { RootState } from '../../shared/store/domain';
import {
  AppConfiguration,
  AppConfigurationExtended,
  WegApplicationOptions,
  WmApplicationOptions,
} from '../domain';

import { IdWithIdentifier } from '../../../../shared/schemas/AppsConfigurations';
import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../../components/SettingsBox';
import { Identifier } from './Identifier';
import cs from './index.module.css';

interface Props {
  idx?: number;
  open: boolean;
  onSave: (app: AppConfigurationExtended) => void;
  onCancel: () => void;
  isNew?: boolean;
  readonlyApp?: AppConfigurationExtended;
}

const getAppSelector = (idx: number | undefined, isNew: boolean) =>
  createSelector([ownSelector], (state: RootState) => {
    return idx != null && !isNew ? state.appsConfigurations[idx]! : AppConfiguration.default();
  });

export const EditAppModal = ({ idx, onCancel, onSave, isNew, open, readonlyApp }: Props) => {
  const { t } = useTranslation();

  const monitors = useSelector(RootSelectors.monitors);
  const _app = useSelector(getAppSelector(idx, !!isNew));
  const initialState = readonlyApp || _app;
  const isReadonly = !!readonlyApp;

  const [app, setApp] = useState(initialState);

  useEffect(() => {
    if (isNew && !open) {
      // reset state on close
      setApp(initialState);
    }
  }, [open]);

  const onInternalSave = () => {
    onSave(app as AppConfigurationExtended);
  };

  const updateName = (e: React.ChangeEvent<HTMLInputElement>) =>
    setApp({ ...app, name: e.target.value });
  const updateCategory = (e: React.ChangeEvent<HTMLInputElement>) =>
    setApp({ ...app, category: e.target.value || null });

  const onChangeIdentifier = (identifier: IdWithIdentifier) => setApp({ ...app, identifier });

  const onSelectMonitor = (value: number | null) => setApp({ ...app, monitor: value });
  const onSelectWorkspace = (value: string | null) => setApp({ ...app, workspace: value });

  const onChangeOption = (option: WmApplicationOptions | WegApplicationOptions, checked: boolean) => {
    setApp({ ...app, options: checked ? [...app.options, option] : app.options.filter((o) => o !== option) });
  };

  const monitorsOptions = monitors.map((_, i) => ({ label: `Monitor ${i + 1}`, value: i }));
  const workspaceOptions =
    app.monitor != null && monitors[app.monitor]
      ? monitors[app.monitor]?.workspaces.map(({ name }) => ({ label: name, value: name }))
      : [];

  let title = t('apps_configurations.app.title_edit');
  let okText = t('apps_configurations.app.ok_edit');

  if (isNew) {
    title = t('apps_configurations.app.title_create');
    okText = t('apps_configurations.app.ok_create');
  }

  if (isReadonly) {
    title = t('apps_configurations.app.title_readonly');
    okText = t('apps_configurations.app.ok_readonly');
  }

  return (
    <Modal
      title={title.replace('{{name}}', app.name)}
      open={open}
      onCancel={onCancel}
      onOk={onInternalSave}
      cancelText={t('cancel')}
      okText={okText}
      cancelButtonProps={isReadonly ? { style: { display: 'none' } } : undefined}
      centered
      className={cs.editModal}
    >
      <ConfigProvider componentDisabled={isReadonly}>
        {!!readonlyApp && (
          <SettingsGroup>
            <b>{t('apps_configurations.bundled_title')}</b>
            <p>{t('apps_configurations.bundled_msg')}</p>
          </SettingsGroup>
        )}

        <SettingsGroup>
          <div>
            <SettingsOption>
              <span>{t('apps_configurations.app.name')}</span>
              <Input value={app.name} onChange={updateName} required />
            </SettingsOption>
            <SettingsOption>
              <span>{t('apps_configurations.app.category')}</span>
              <Input
                value={app.category || ''}
                placeholder={t('apps_configurations.app.category_placeholder')}
                onChange={updateCategory}
              />
            </SettingsOption>
          </div>
        </SettingsGroup>

        <Identifier identifier={app.identifier} onChange={onChangeIdentifier} />

        <SettingsGroup>
          <SettingsSubGroup label={t('apps_configurations.app.bindings')}>
            <SettingsOption>
              <span>{t('apps_configurations.app.monitor')}</span>
              <Select
                value={app.monitor}
                placeholder={t('apps_configurations.app.monitor_placeholder')}
                allowClear
                options={monitorsOptions}
                onChange={onSelectMonitor}
                disabled // Todo(eythan) remove when enable monitors on release
              />
            </SettingsOption>
            <SettingsOption>
              <span>{t('apps_configurations.app.workspace')}</span>
              <Select
                value={app.workspace}
                placeholder={t('apps_configurations.app.workspace_placeholder')}
                allowClear
                options={workspaceOptions}
                onChange={onSelectWorkspace}
                disabled // Todo(eythan) remove when enable monitors on release
              />
            </SettingsOption>
          </SettingsSubGroup>
        </SettingsGroup>

        <SettingsGroup>
          <SettingsSubGroup label={t('apps_configurations.app.wm_options_label')}>
            {Object.values(WmApplicationOptions).map((value, i) => (
              <SettingsOption key={i}>
                <span>{t(`apps_configurations.app.options.${value}`)}</span>
                <Switch value={app.options.includes(value)} onChange={onChangeOption.bind(this, value)} />
              </SettingsOption>
            ))}
          </SettingsSubGroup>

          <SettingsSubGroup label={t('apps_configurations.app.weg_options_label')}>
            {Object.values(WegApplicationOptions).map((value, i) => (
              <SettingsOption key={i}>
                <span>{t(`apps_configurations.app.options.${value}`)}</span>
                <Switch value={app.options.includes(value)} onChange={onChangeOption.bind(this, value)} />
              </SettingsOption>
            ))}
          </SettingsSubGroup>
        </SettingsGroup>
      </ConfigProvider>
    </Modal>
  );
};
