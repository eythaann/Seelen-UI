import { SettingsGroup } from '../../../components/SettingsBox';

import { BorderSettings } from '../border/infra';
import { ContainerTopBarSettings } from '../containerTopBar/infra';
import { PopupsSettings } from '../popups/infra';

export const StylesView = () => {
  return (
    <>
      <PopupsSettings />
      <ContainerTopBarSettings />
      <SettingsGroup>
        <BorderSettings />
      </SettingsGroup>
    </>
  );
};
