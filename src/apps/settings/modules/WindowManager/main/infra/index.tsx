import { SettingsGroup, SettingsOption } from '../../../../components/SettingsBox';
import { GlobalPaddings } from './GlobalPaddings';
import { OthersConfigs } from './Others';
import { Select, Switch } from 'antd';
import { useDispatch, useSelector } from 'react-redux';

import { BorderSettings } from '../../border/infra';
import { ContainerTopBarSettings } from '../../containerTopBar/infra';

import { newSelectors } from '../../../shared/store/app/reducer';
import { RootSelectors } from '../../../shared/store/app/selectors';
import { WManagerSettingsActions } from '../app';

export function WindowManagerSettings() {
  const settings = useSelector(RootSelectors.windowManager);
  const layouts = useSelector(newSelectors.availableLayouts);
  const defaultLayout = useSelector(newSelectors.windowManager.defaultLayout);

  const dispatch = useDispatch();

  const onToggleEnable = (value: boolean) => {
    dispatch(WManagerSettingsActions.setEnabled(value));
  };

  const onSelectLayout = (value: string) => {
    dispatch(WManagerSettingsActions.setDefaultLayout(value));
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <div>
            <b>Enable Tiling Window Manager</b>
          </div>
          <Switch checked={settings.enabled} onChange={onToggleEnable} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <div>
            <b>Default Layout: </b>
          </div>
          <Select
            style={{ width: '200px' }}
            value={defaultLayout}
            options={layouts.map((layout) => ({
              label: layout.info.displayName,
              value: layout.info.filename,
            }))}
            onSelect={onSelectLayout}
          />
        </SettingsOption>
      </SettingsGroup>

      <ContainerTopBarSettings />
      <GlobalPaddings />
      <OthersConfigs />
      <SettingsGroup>
        <BorderSettings />
      </SettingsGroup>
    </>
  );
}
