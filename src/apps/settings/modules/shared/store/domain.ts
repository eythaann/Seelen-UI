import { AppTemplate, Theme } from '../../../../../shared.interfaces';
import { SeelenWegState } from '../../../../utils/interfaces/Weg';
import { Route } from '../../../components/navigation/routes';

import { AppConfiguration } from '../../appsConfigurations/domain';
import { Monitor } from '../../monitors/main/domain';
import { SeelenManagerState } from '../../WindowManager/main/domain';

export interface RootState {
  route: Route;
  toBeSaved: boolean;
  monitors: Monitor[];
  appsConfigurations: AppConfiguration[];
  appsTemplates: (Omit<AppTemplate, 'apps'> & { apps: AppConfiguration[] })[];
  windowManager: SeelenManagerState;
  seelenweg: SeelenWegState;
  availableThemes: Theme[];
  theme: Theme | null;
  selectedTheme: string | null;
  ahkEnabled: boolean;
  autostart: boolean;
}