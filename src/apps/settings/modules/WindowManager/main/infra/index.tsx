import { SettingsGroup } from '../../../../components/SettingsBox';
import { GlobalPaddings } from './GlobalPaddings';
import { OthersConfigs } from './Others';

import { BorderSettings } from '../../border/infra';
import { ContainerTopBarSettings } from '../../containerTopBar/infra';

export function WindowManagerSettings() {
  return (
    <>
      <ContainerTopBarSettings />
      <GlobalPaddings />
      <OthersConfigs />
      <SettingsGroup>
        <BorderSettings />
      </SettingsGroup>
    </>
  );
}
