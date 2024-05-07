import { SettingsGroup, SettingsOption } from '../../components/SettingsBox';
import { Switch, Tooltip } from 'antd';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../shared/utils/infra';

import { RootActions } from '../shared/store/app/reducer';
import { RootSelectors } from '../shared/store/app/selectors';

export function Shortcuts() {
  const ahkEnable = useAppSelector(RootSelectors.ahkEnabled);

  const dispatch = useDispatch();

  const onChange = (value: boolean) => {
    dispatch(RootActions.setAhkEnabled(value));
    dispatch(RootActions.setToBeSaved(true));
  };

  return (
    <div>
      <SettingsGroup>
        <SettingsOption>
          <span>
            Enable Seelen UI shortcuts{' '}
            <Tooltip
              title="Disable if you will implement your own shortcuts using the CLI."
            >
              🛈
            </Tooltip>
          </span>
          <Switch value={ahkEnable} onChange={onChange} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <div>Configurable shortcuts using UI is in progress</div>
        </SettingsOption>
      </SettingsGroup>
    </div>
  );
}
