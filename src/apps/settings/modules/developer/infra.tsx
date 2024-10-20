import { path } from '@tauri-apps/api';
import { invoke } from '@tauri-apps/api/core';
import { Button, Switch } from 'antd';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';
import { SeelenCommand } from 'seelen-core';

import { resolveDataPath } from '../shared/config/infra';

import { newSelectors, RootActions } from '../shared/store/app/reducer';
import { LoadCustomConfigFile } from './app';

import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../components/SettingsBox';

export function DeveloperTools() {
  const devTools = useSelector(newSelectors.devTools);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  function onToggleDevTools(value: boolean) {
    dispatch(RootActions.setDevTools(value));
  }

  async function openSettingsFile() {
    invoke(SeelenCommand.OpenFile, { path: await resolveDataPath('settings.json') });
  }

  async function openInstallFolder() {
    invoke(SeelenCommand.OpenFile, { path: await path.resourceDir() });
  }

  async function openDataFolder() {
    invoke(SeelenCommand.OpenFile, { path: await path.appDataDir() });
  }

  async function simulateFullscreen(value: boolean) {
    invoke(SeelenCommand.SimulateFullscreen, { value });
  }

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>{t('devtools.enable')}</b>
          <Switch value={devTools} onChange={onToggleDevTools} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t('devtools.app_folders')}>
          <SettingsOption>
            <span>{t('devtools.install_folder')}</span>
            <Button onClick={openInstallFolder}>{t('open')}</Button>
          </SettingsOption>
          <SettingsOption>
            <span>{t('devtools.data_folder')}</span>
            <Button onClick={openDataFolder}>{t('open')}</Button>
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <span>{t('devtools.settings_file')}</span>
          <Button onClick={openSettingsFile}>{t('open')}</Button>
        </SettingsOption>
        <SettingsOption>
          <span>{t('devtools.custom_config_file')}:</span>
          <Button onClick={LoadCustomConfigFile}>{t('devtools.load')}</Button>
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <span>{t('devtools.simulate_fullscreen')}</span>
          <Switch onChange={simulateFullscreen} />
        </SettingsOption>
      </SettingsGroup>
    </>
  );
}
