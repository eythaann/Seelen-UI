import { Icon } from '../../../shared/components/Icon';
import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../components/SettingsBox';
import { invoke } from '@tauri-apps/api/core';
import { exit, relaunch } from '@tauri-apps/plugin-process';
import { Button } from 'antd';

import { EnvConfig, getSettingsPath } from '../shared/config/infra';
import { LoadCustomConfigFile } from './infra.api';
import cs from './infra.module.css';

export function Information() {
  const openSettingsFile = async () => {
    invoke('open_file', { path: await getSettingsPath('settings.json') });
  };

  return (
    <div className={cs.info}>
      <SettingsGroup>
        <SettingsSubGroup label="Links:">
          <SettingsOption>
            <span>
              Seelen UI <span className={cs.version}>v{EnvConfig.version}</span>:
            </span>
            <a href="https://github.com/eythaann/seelen-ui" target="_blank">
              github.com/eythaann/seelen-ui
            </a>
          </SettingsOption>
          <SettingsOption>
            <span>Discord:</span>
            <a href="https://discord.gg/ABfASx5ZAJ" target="_blank">
              discord.gg/ABfASx5ZAJ
            </a>
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

      <SettingsGroup>
        <SettingsOption>
          <span>Force Restart</span>
          <Button type="dashed" onClick={relaunch} style={{ width: '50px' }}>
            <Icon iconName="IoReload" propsIcon={{ size: 12 }} />
          </Button>
        </SettingsOption>
        <SettingsOption>
          <span>Quit/Close Seelen UI</span>
          <Button type="dashed" danger onClick={() => exit(0)} style={{ width: '50px' }}>
            <Icon iconName="IoClose" />
          </Button>
        </SettingsOption>
      </SettingsGroup>
    </div>
  );
}
