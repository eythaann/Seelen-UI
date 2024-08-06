import {
  SettingsGroup,
  SettingsOption,
} from '../../../../components/SettingsBox';
import { invoke } from '@tauri-apps/api/core';
import { ColorPicker } from 'antd';
import { useSelector } from 'react-redux';

import { newSelectors } from '../../../shared/store/app/reducer';

import cs from './index.module.css';

export function Colors() {
  const colors = useSelector(newSelectors.colors);

  return (
    <SettingsGroup>
      <SettingsOption>
        <b>Accent Color</b>
        <div
          onClickCapture={(e) => {
            e.stopPropagation();
            invoke('open_file', { path: 'ms-settings:colors' }).catch(console.error);
          }}
        >
          <ColorPicker value={colors.accent} disabledAlpha showText />
        </div>
      </SettingsOption>
      <div className={cs.palette}>
        <div style={{ backgroundColor: colors.accent_darkest }} />
        <div style={{ backgroundColor: colors.accent_darker }} />
        <div style={{ backgroundColor: colors.accent_dark }} />
        <div style={{ backgroundColor: colors.accent }} />
        <div style={{ backgroundColor: colors.accent_light }} />
        <div style={{ backgroundColor: colors.accent_lighter }} />
        <div style={{ backgroundColor: colors.accent_lightest }} />
      </div>
    </SettingsGroup>
  );
}
