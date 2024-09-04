import { Layout } from '../../../../shared/schemas/Layout';
import { Placeholder } from '../../../../shared/schemas/Placeholders';
import { Route } from '../../../components/navigation/routes';
import { Settings, Theme, UIColors } from 'seelen-core';

import { AppConfiguration } from '../../appsConfigurations/domain';

export interface RootState extends Settings {
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
