import { Icon } from '../../../shared/components/Icon';
import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../components/SettingsBox';
import { exit, relaunch } from '@tauri-apps/plugin-process';
import { Button, Switch } from 'antd';
import { useDispatch, useSelector } from 'react-redux';

import { EnvConfig } from '../shared/config/infra';
import cs from './infra.module.css';

import { newSelectors, RootActions } from '../shared/store/app/reducer';

export function Information() {
  const devTools = useSelector(newSelectors.devTools);

  const dispatch = useDispatch();

  function onToggleDevTools(value: boolean) {
    dispatch(RootActions.setDevTools(value));
  }

  return (
    <div className={cs.info}>
      <SettingsGroup>
        <SettingsSubGroup label="Seelen UI">
          <SettingsOption>
            <span>Version:</span>
            <span className={cs.version}>v{EnvConfig.version}</span>
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label="Official Links">
          <SettingsOption>
            <span>Github:</span>
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
          <span>Development Mode</span>
          <Switch value={devTools} onChange={onToggleDevTools} />
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
          <span>Quit/Close</span>
          <Button type="dashed" danger onClick={() => exit(0)} style={{ width: '50px' }}>
            <Icon iconName="IoClose" />
          </Button>
        </SettingsOption>
      </SettingsGroup>
    </div>
  );
}
