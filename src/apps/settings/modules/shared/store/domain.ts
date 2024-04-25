import { AppTemplate } from '../../../../../shared.interfaces';
import { ISettings } from '../../../../utils/schemas/Settings';
import { Theme } from '../../../../utils/schemas/Theme';
import { Route } from '../../../components/navigation/routes';

import { AppConfiguration } from '../../appsConfigurations/domain';

export interface RootState extends ISettings {
  route: Route;
  toBeSaved: boolean;
  appsConfigurations: AppConfiguration[];
  appsTemplates: (Omit<AppTemplate, 'apps'> & { apps: AppConfiguration[] })[];
  theme: Theme | null;
  availableThemes: Theme[];
  selectedTheme: string | null;
  autostart: boolean;
}