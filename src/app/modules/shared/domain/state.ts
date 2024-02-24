import { AppTemplate } from '../../../../shared.interfaces';
import { Route } from './routes';

import { AppConfiguration } from '../../appsConfigurations/domain';
import { GeneralSettingsState } from '../../general/main/domain';
import { Monitor } from '../../monitors/main/domain';

export interface RootState {
  route: Route;
  toBeSaved: boolean;
  generals: GeneralSettingsState;
  monitors: Monitor[];
  appsConfigurations: AppConfiguration[];
  appsTemplates: (Omit<AppTemplate, 'apps'> & { apps: AppConfiguration[] })[];
  ahkEnabled: boolean;
}