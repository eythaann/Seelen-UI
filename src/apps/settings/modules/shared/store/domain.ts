import { Layout } from '../../../../shared/schemas/Layout';
import { Placeholder } from '../../../../shared/schemas/Placeholders';
import { ISettings } from '../../../../shared/schemas/Settings';
import { Theme } from '../../../../shared/schemas/Theme';
import { Route } from '../../../components/navigation/routes';

import { AppConfiguration } from '../../appsConfigurations/domain';

export interface UIColors {
  background: string;
  foreground: string;
  accent_darkest: string;
  accent_darker: string;
  accent_dark: string;
  accent: string;
  accent_light: string;
  accent_lighter: string;
  accent_lightest: string;
  complement: string | null;
}

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
  colors: UIColors;
  wallpaper: string | null;
}
