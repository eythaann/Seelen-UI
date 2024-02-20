import {
  SettingsGroup,
  SettingsOption,
  SettingsSubGroup,
} from '../../../../components/SettingsBox';
import { ContainerBehaviors } from './ContainerBehaviours';
import { FocusBehaviours } from './FocusBehaviours';
import { GlobalPaddings } from './GlobalPaddings';
import { OthersConfigs } from './Others';
import { InputNumber, Select, Switch } from 'antd';
import { useEffect, useState } from 'react';
import { useSelector } from 'react-redux';

import { AnimationsSettings } from '../../animations/infra';
import { BorderSettings } from '../../border/infra';
import { ContainerTopBarSettings } from '../../containerTopBar/infra';

export function General() {
  const [autostart, setAutostart] = useState(false);

  useEffect(() => {
    window.backgroundApi.autostartTaskExist().then((value) => setAutostart(value));
  }, []);

  const onAutoStart = (value: boolean) => {
    if (value) {
      window.backgroundApi.enableAutostart();
      setAutostart(true);
    } else {
      window.backgroundApi.disableAutostart();
      setAutostart(false);
    }
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <span style={{ fontWeight: 600 }}>Run Komorebi-UI at startup?</span>
          <Switch onChange={onAutoStart} value={autostart} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <BorderSettings />
        <FocusBehaviours />
      </SettingsGroup>

      <ContainerBehaviors />
      <AnimationsSettings />
      <GlobalPaddings/>
      <ContainerTopBarSettings />

      <OthersConfigs />
    </>
  );
}
