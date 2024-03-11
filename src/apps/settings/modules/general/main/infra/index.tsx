import {
  SettingsGroup,
  SettingsOption,
} from '../../../../components/SettingsBox';
import { ContainerBehaviors } from './ContainerBehaviours';
import { FocusBehaviours } from './FocusBehaviours';
import { GlobalPaddings } from './GlobalPaddings';
import { OthersConfigs } from './Others';
import * as autostart from '@tauri-apps/plugin-autostart';
import { Switch } from 'antd';
import { useEffect, useState } from 'react';

import { AnimationsSettings } from '../../animations/infra';

export function General() {
  const [autostartStatus, setAutostartStatus] = useState(false);

  useEffect(() => {
    autostart.isEnabled().then((value) => setAutostartStatus(value));
  }, []);

  const onAutoStart = (value: boolean) => {
    if (value) {
      autostart.enable().then(() => setAutostartStatus(true));
    } else {
      autostart.disable().then(() => setAutostartStatus(false));
    }
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <span style={{ fontWeight: 600 }}>Run Komorebi-UI at startup?</span>
          <Switch onChange={onAutoStart} value={autostartStatus} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <FocusBehaviours />
      </SettingsGroup>

      <ContainerBehaviors />
      <AnimationsSettings />
      <GlobalPaddings/>

      <OthersConfigs />
    </>
  );
}
