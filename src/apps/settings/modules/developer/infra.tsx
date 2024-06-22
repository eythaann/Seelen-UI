import { SettingsGroup, SettingsOption } from '../../components/SettingsBox';
import { invoke } from '@tauri-apps/api/core';
import { Button, Switch } from 'antd';
import { useDispatch, useSelector } from 'react-redux';

import { resolveDotConfigPath } from '../shared/config/infra';

import { newSelectors, RootActions } from '../shared/store/app/reducer';
import { LoadCustomConfigFile } from './app';

export function DeveloperTools() {
  const devTools = useSelector(newSelectors.devTools);

  const dispatch = useDispatch();

  function onToggleDevTools(value: boolean) {
    dispatch(RootActions.setDevTools(value));
  }

  async function openSettingsFile() {
    invoke('open_file', { path: await resolveDotConfigPath('settings.json') });
  }

  async function openInstallFolder() {
    invoke('open_install_folder');
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
        <SettingsOption>
          <span>Open Install Folder</span>
          <Button onClick={openInstallFolder}>Open</Button>
        </SettingsOption>
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
