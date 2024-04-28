import { Icon } from '../../../utils/components/Icon';
import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../components/SettingsBox';
import { exit, relaunch } from '@tauri-apps/plugin-process';
import { Button } from 'antd';

import { EnvConfig } from '../shared/config/infra';
import { LoadCustomConfigFile } from './infra.api';
import cs from './infra.module.css';

export function Information() {
  return (
    <div className={cs.info}>
      <SettingsGroup>
        <SettingsSubGroup label="Documentation">
          <SettingsOption>
            <span>
              Seelen UI <span className={cs.version}>v{EnvConfig.version}</span>:
            </span>
            <a href="https://github.com/eythaann/seelen-ui" target="_blank">
              github.com/eythaann/seelen-ui
            </a>
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label="Follow me:">
          <SettingsOption>
            <span>Github:</span>
            <a href="https://github.com/eythaann" target="_blank">
              github.com/eythaann
            </a>
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <span>Load config file (will replace current configurations):</span>
          <Button onClick={LoadCustomConfigFile}>Select File</Button>
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <span>Force Restart</span>
          <Button type="dashed" onClick={relaunch} style={{ width: '40px' }}>
            <Icon lib="io5" iconName="IoReload" propsIcon={{ size: 12 }} />
          </Button>
        </SettingsOption>
        <SettingsOption>
          <span>Quit/Close</span>
          <Button type="dashed" onClick={() => exit(0)} style={{ width: '40px' }}>
            <Icon lib="io5" iconName="IoClose" />
          </Button>
        </SettingsOption>
      </SettingsGroup>
    </div>
  );
}
