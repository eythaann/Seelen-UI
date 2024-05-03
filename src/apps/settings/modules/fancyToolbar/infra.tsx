import { SettingsGroup, SettingsOption } from '../../components/SettingsBox';
import { Select, Switch } from 'antd';
import { useDispatch, useSelector } from 'react-redux';

import { newSelectors } from '../shared/store/app/reducer';
import { RootSelectors } from '../shared/store/app/selectors';
import { FancyToolbarActions } from './app';

export function FancyToolbarSettings() {
  const settings = useSelector(RootSelectors.fancyToolbar);
  const placeholders = useSelector(newSelectors.availablePlaceholders);
  const selectedStructure = useSelector(newSelectors.fancyToolbar.placeholder);

  const dispatch = useDispatch();

  const onToggleEnable = (value: boolean) => {
    dispatch(FancyToolbarActions.setEnabled(value));
  };

  const onSelectStructure = (value: string) => {
    dispatch(FancyToolbarActions.setPlaceholder(value));
  };

  const usingStructure = placeholders.find((placeholder) => placeholder.info.filename === selectedStructure);

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

      <SettingsGroup>
        <SettingsOption>
          <div>
            <b>Default Layout: </b>
          </div>
          <Select
            style={{ width: '200px' }}
            value={selectedStructure}
            options={placeholders.map((placeholder) => ({
              label: placeholder.info.displayName,
              value: placeholder.info.filename,
            }))}
            onSelect={onSelectStructure}
          />
        </SettingsOption>
        <div>
          <p>
            <b>Author: </b>{usingStructure?.info.author}
          </p>
          <p><b>Description: </b>{usingStructure?.info.description}</p>
        </div>
      </SettingsGroup>
    </>
  );
}
