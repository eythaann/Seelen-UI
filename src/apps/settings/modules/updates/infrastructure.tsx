/** Unused File but it is still here for maybe a future use */
import { SettingsGroup, SettingsOption } from '../../components/SettingsBox';
import { Switch } from 'antd';
import { useDispatch } from 'react-redux';

import cs from './infra.module.css';

import { useAppSelector } from '../shared/app/hooks';
import { RootActions } from '../shared/app/reducer';
import { RootSelectors } from '../shared/app/selectors';

export function Updates() {
  const ahkEnable = useAppSelector(RootSelectors.updateNotification);

  const dispatch = useDispatch();

  const onChange = (value: boolean) => {
    dispatch(RootActions.setUpdateNotification(value));
    dispatch(RootActions.setToBeSaved(true));
  };

  return (
    <div className={cs.info}>
      <SettingsGroup>
        <SettingsOption>
          <span style={{ fontWeight: 600 }}>Enable Auto-Update Notification</span>
          <Switch value={ahkEnable} onChange={onChange} />
        </SettingsOption>
      </SettingsGroup>
      <SettingsGroup>
        <div className={cs.disclaimer}>
          By default auto update notification is disabled. This is because it is detected as a virus
          by anti-virus software for some users. If you will enable it, add Seelen UI to excluded
          programs in your anti-virus software settings to avoid false positives.
        </div>
        <SettingsOption>
          You can manually download the latest version from:{' '}
          <a href="https://github.com/eythaann/seelen-ui/releases" target="_blank">Github Releases</a>
        </SettingsOption>
      </SettingsGroup>
    </div>
  );
}
