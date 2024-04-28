import { SettingsGroup, SettingsOption } from '../../components/SettingsBox';
import { Switch } from 'antd';
import { useDispatch, useSelector } from 'react-redux';

import { RootSelectors } from '../shared/store/app/selectors';
import { FancyToolbarActions } from './app';

export function FancyToolbarSettings() {
  const settings = useSelector(RootSelectors.fancyToolbar);

  const dispatch = useDispatch();

  const onToggleEnable = (value: boolean) => {
    dispatch(FancyToolbarActions.setEnabled(value));
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <div>
            <b>Enable Fancy Toolbar (Beta)</b>
          </div>
          <Switch checked={settings.enabled} onChange={onToggleEnable} />
        </SettingsOption>
      </SettingsGroup>
    </>
  );
}
