import { SettingsGroup, SettingsOption } from '../../components/SettingsBox';
import { Select } from 'antd';

import { GeneralSettingsActions } from '../general/main/app';
import { useAppDispatch, useAppSelector } from '../shared/app/hooks';
import { RootActions } from '../shared/app/reducer';
import { GeneralSettingsSelectors, RootSelectors } from '../shared/app/selectors';

export const ThemesView = () => {
  const selectedTheme = useAppSelector(GeneralSettingsSelectors.selectedTheme);
  const themes = useAppSelector(RootSelectors.availableThemes);
  const usingTheme = useAppSelector(RootSelectors.theme);

  const dispatch = useAppDispatch();

  const onSelectTheme = (theme: string) => {
    dispatch(GeneralSettingsActions.setSelectedTheme(theme));
    dispatch(RootActions.setTheme(themes.find((t) => t.info.filename === theme)!));
  };

  return (
    <>
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
      {/* <PopupsSettings />
      <ContainerTopBarSettings />
      <SettingsGroup>
        <BorderSettings />
      </SettingsGroup> */}
    </>
  );
};
