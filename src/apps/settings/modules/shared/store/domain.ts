import { AppTemplate } from '../../../../../shared.interfaces';
import { Layout } from '../../../../utils/schemas/Layout';
import { Placeholder } from '../../../../utils/schemas/Placeholders';
import { ISettings } from '../../../../utils/schemas/Settings';
import { Theme } from '../../../../utils/schemas/Theme';
import { Route } from '../../../components/navigation/routes';

import { AppConfiguration } from '../../appsConfigurations/domain';

export interface RootState extends ISettings {
  route: Route;
  toBeSaved: boolean;
  appsConfigurations: AppConfiguration[];
  appsTemplates: (Omit<AppTemplate, 'apps'> & { apps: AppConfiguration[] })[];
  selectedTheme: string | null;
  availableThemes: Theme[];
  availableLayouts: Layout[];
  availablePlaceholders: Placeholder[];
  autostart: boolean;
}