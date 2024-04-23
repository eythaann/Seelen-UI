import { SettingsGroup, SettingsOption } from '../../../../components/SettingsBox';
import { Select, Switch } from 'antd';
import { useSelector } from 'react-redux';

import { startup } from '../../../shared/infrastructure/tauri';

import { useAppDispatch } from '../../../shared/app/hooks';
import { RootActions } from '../../../shared/app/reducer';
import { GeneralSettingsSelectors, RootSelectors } from '../../../shared/app/selectors';
import { GeneralSettingsActions } from '../app';

export function General() {
  const autostartStatus = useSelector(RootSelectors.autostart);
  const selectedTheme = useSelector(GeneralSettingsSelectors.selectedTheme);
  const themes = useSelector(RootSelectors.availableThemes);
  const usingTheme = useSelector(RootSelectors.theme);

  const dispatch = useAppDispatch();

  const onSelectTheme = (theme: string) => {
    dispatch(GeneralSettingsActions.setSelectedTheme(theme));
    dispatch(RootActions.setTheme(themes.find((t) => t.info.filename === theme)!));
  };

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
      <SettingsGroup>
        <SettingsOption>
          <div>
            <b>Theme: </b>
          </div>
          <Select
            style={{ width: '200px' }}
            value={selectedTheme}
            options={themes.map((theme) => ({
              label: theme.info.displayName,
              value: theme.info.filename,
            }))}
            onSelect={onSelectTheme}
          />
        </SettingsOption>
        <div>
          <p>
            <b>Author: </b>{usingTheme?.info.author}
          </p>
          <p><b>Description: </b>{usingTheme?.info.description}</p>
        </div>
      </SettingsGroup>
    </>
  );
}
