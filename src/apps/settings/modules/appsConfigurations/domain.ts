import { AppExtraFlag } from '@seelen-ui/lib';
import { AppConfig } from '@seelen-ui/lib/types';

export const WmApplicationOptions = [
  AppExtraFlag.Float,
  AppExtraFlag.Unmanage,
  AppExtraFlag.Force,
  AppExtraFlag.Pinned,
];

export const WegApplicationOptions = [AppExtraFlag.Hidden];

export interface AppConfigurationExtended extends AppConfig {
  key: number;
}
