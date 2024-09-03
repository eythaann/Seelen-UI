import { UIColors } from '../../../../../../lib/src/system_state';
import { Layout } from '../../../../shared/schemas/Layout';
import { Placeholder } from '../../../../shared/schemas/Placeholders';
import { ISettings } from '../../../../shared/schemas/Settings';
import { Theme } from '../../../../shared/schemas/Theme';
import { Route } from '../../../components/navigation/routes';

import { AppConfiguration } from '../../appsConfigurations/domain';

export interface RootState extends ISettings {
  lastLoaded: this | null;
  route: Route;
  toBeSaved: boolean;
  toBeRestarted: boolean;
  appsConfigurations: AppConfiguration[];
  availableThemes: Theme[];
  availableLayouts: Layout[];
  availablePlaceholders: Placeholder[];
  autostart: boolean | null;
  wallpaper: string | null;
  colors: UIColors;
}
