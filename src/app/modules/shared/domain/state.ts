import { Route } from './routes';

import { GeneralSettingsState } from '../../general/main/domain';

export interface GlobalState {
  route: Route;
  generals: GeneralSettingsState;
}