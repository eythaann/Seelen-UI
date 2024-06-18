import { SettingsGroup, SettingsOption } from '../../../../components/SettingsBox';
import { GlobalPaddings } from './GlobalPaddings';
import { OthersConfigs } from './Others';
import { Select, Switch } from 'antd';
import { useDispatch, useSelector } from 'react-redux';

import { BorderSettings } from '../../border/infra';

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

  const usingLayout = layouts.find((layout) => layout.info.filename === defaultLayout);

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
            options={layouts.map((layout, idx) => ({
              key: `layout-${idx}`,
              label: layout.info.displayName,
              value: layout.info.filename,
            }))}
            onSelect={onSelectLayout}
          />
        </SettingsOption>
        <div>
          <p>
            <b>Author: </b>
            {usingLayout?.info.author}
          </p>
          <p>
            <b>Description: </b>
            {usingLayout?.info.description}
          </p>
        </div>
      </SettingsGroup>

      <GlobalPaddings />
      <OthersConfigs />
      <SettingsGroup>
        <BorderSettings />
      </SettingsGroup>
    </>
  );
}
