import { SettingsGroup, SettingsOption } from '../../../../components/SettingsBox';
import { Switch } from 'antd';
import { useSelector } from 'react-redux';

import { startup } from '../../../shared/infrastructure/tauri';

import { useAppDispatch } from '../../../shared/app/hooks';
import { RootActions } from '../../../shared/app/reducer';
import { RootSelectors } from '../../../shared/app/selectors';

export function General() {
  const autostartStatus = useSelector(RootSelectors.autostart);

  const dispatch = useAppDispatch();

  const onAutoStart = async (value: boolean) => {
    if (value) {
      await startup.enable();
    } else {
      await startup.disable();
    }
    dispatch(RootActions.setAutostart(value));
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <span style={{ fontWeight: 600 }}>Run Seelen UI at startup?</span>
          <Switch onChange={onAutoStart} value={autostartStatus} />
        </SettingsOption>
      </SettingsGroup>
    </>
  );
}
