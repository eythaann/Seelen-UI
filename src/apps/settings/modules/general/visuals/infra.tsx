import { SettingsGroup, SettingsOption } from '../../../components/SettingsBox';
import { Select } from 'antd';

import { BorderSettings } from '../border/infra';
import { ContainerTopBarSettings } from '../containerTopBar/infra';
import { PopupsSettings } from '../popups/infra';

import { useAppDispatch, useAppSelector } from '../../shared/app/hooks';
import { RootActions } from '../../shared/app/reducer';
import { GeneralSettingsSelectors, RootSelectors } from '../../shared/app/selectors';
import { GeneralSettingsActions } from '../main/app';

export const StylesView = () => {
  const selectedTheme = useAppSelector(GeneralSettingsSelectors.selectedTheme);
  const themes = useAppSelector(RootSelectors.availableThemes);

  const dispatch = useAppDispatch();

  const onSelectTheme = (theme: string) => {
    dispatch(GeneralSettingsActions.setSelectedTheme(theme));
    dispatch(RootActions.setTheme(themes.find((t) => t.info.filename === theme)!));
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>Theme 🎨</b>
          <Select
            value={selectedTheme}
            options={themes.map((theme) => ({ label: theme.info.displayName, value: theme.info.filename }))}
            onSelect={onSelectTheme}
          />
        </SettingsOption>
      </SettingsGroup>
      <PopupsSettings />
      <ContainerTopBarSettings />
      <SettingsGroup>
        <BorderSettings />
      </SettingsGroup>
    </>
  );
};
