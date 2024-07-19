import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../components/SettingsBox';
import { path } from '@tauri-apps/api';
import { invoke } from '@tauri-apps/api/core';
import { Button, Switch } from 'antd';
import { useDispatch, useSelector } from 'react-redux';

import { resolveDataPath } from '../shared/config/infra';

import { newSelectors, RootActions } from '../shared/store/app/reducer';
import { LoadCustomConfigFile } from './app';

export function DeveloperTools() {
  const devTools = useSelector(newSelectors.devTools);

  const dispatch = useDispatch();

  function onToggleDevTools(value: boolean) {
    dispatch(RootActions.setDevTools(value));
  }

  async function openSettingsFile() {
    invoke('open_file', { path: await resolveDataPath('settings.json') });
  }

  async function openInstallFolder() {
    invoke('open_file', { path: await path.resourceDir() });
  }

  async function openDataFolder() {
    invoke('open_file', { path: await path.appDataDir() });
  }

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>Developer Mode</b>
          <Switch value={devTools} onChange={onToggleDevTools} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label="App Folders">
          <SettingsOption>
            <span>Install Folder</span>
            <Button onClick={openInstallFolder}>Open</Button>
          </SettingsOption>
          <SettingsOption>
            <span>Data Folder</span>
            <Button onClick={openDataFolder}>Open</Button>
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <span>Open Settings file</span>
          <Button onClick={openSettingsFile}>Open</Button>
        </SettingsOption>
        <SettingsOption>
          <span>Load Custom file (will replace current settings):</span>
          <Button onClick={LoadCustomConfigFile}>Select File</Button>
        </SettingsOption>
      </SettingsGroup>
    </>
  );
}
