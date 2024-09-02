import { wasInstalledUsingMSIX } from '../../../shared';
import { Icon } from '../../../shared/components/Icon';
import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../components/SettingsBox';
import { exit, relaunch } from '@tauri-apps/plugin-process';
import { Button, Switch } from 'antd';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { EnvConfig } from '../shared/config/infra';
import cs from './infra.module.css';

import { newSelectors, RootActions } from '../shared/store/app/reducer';

export function Information() {
  const [isMsixBuild, setIsMsixBuild] = useState(false);

  const devTools = useSelector(newSelectors.devTools);
  const betaChannel = useSelector(newSelectors.betaChannel);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  useEffect(() => {
    wasInstalledUsingMSIX().then(setIsMsixBuild);
  }, []);

  function onToggleDevTools(value: boolean) {
    dispatch(RootActions.setDevTools(value));
  }

  function onToggleBetaChannel(value: boolean) {
    dispatch(RootActions.setBetaChannel(value));
  }

  return (
    <div className={cs.info}>
      <SettingsGroup>
        <SettingsSubGroup label="Seelen UI">
          <SettingsOption>
            <span>{t('extras.version')}:</span>
            <span className={cs.version}>v{EnvConfig.version}</span>
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t('extras.links')}>
          <SettingsOption>
            <span>{t('extras.github')}:</span>
            <a href="https://github.com/eythaann/seelen-ui" target="_blank">
              github.com/eythaann/seelen-ui
            </a>
          </SettingsOption>
          <SettingsOption>
            <span>{t('extras.discord')}:</span>
            <a href="https://discord.gg/ABfASx5ZAJ" target="_blank">
              discord.gg/ABfASx5ZAJ
            </a>
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <span>{t('devtools.enable')}</span>
          <Switch value={devTools} onChange={onToggleDevTools} />
        </SettingsOption>
        <SettingsOption>
          <span>{t('beta_channel.enable')}</span>
          <Switch value={!isMsixBuild && betaChannel} onChange={onToggleBetaChannel} disabled={isMsixBuild}/>
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <span>{t('extras.relaunch')}</span>
          <Button type="dashed" onClick={relaunch} style={{ width: '50px' }}>
            <Icon iconName="IoReload" propsIcon={{ size: 12 }} />
          </Button>
        </SettingsOption>
        <SettingsOption>
          <span>{t('extras.exit')}</span>
          <Button type="dashed" danger onClick={() => exit(0)} style={{ width: '50px' }}>
            <Icon iconName="IoClose" />
          </Button>
        </SettingsOption>
      </SettingsGroup>
    </div>
  );
}
