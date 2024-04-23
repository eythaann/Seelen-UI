import { AppTemplate, Theme } from '../../../../../shared.interfaces';
import { Route } from './routes';

import { AppConfiguration } from '../../appsConfigurations/domain';
import { GeneralSettingsState } from '../../general/main/domain';
import { Monitor } from '../../monitors/main/domain';
import { SeelenWegState } from '../../seelenweg/domain';
import { SeelenManagerState } from '../../WindowManager/main/domain';

export interface RootState {
  route: Route;
  toBeSaved: boolean;
  generals: GeneralSettingsState;
  monitors: Monitor[];
  appsConfigurations: AppConfiguration[];
  appsTemplates: (Omit<AppTemplate, 'apps'> & { apps: AppConfiguration[] })[];
  seelenwm: SeelenManagerState;
  seelenweg: SeelenWegState;
  availableThemes: Theme[];
  theme: Theme | null;
  ahkEnabled: boolean;
  updateNotification: boolean;
  autostart: boolean;
}