import { type AppConfig, AppExtraFlag } from "@seelen-ui/lib/types";

export const WmApplicationOptions = [
  AppExtraFlag.float,
  AppExtraFlag.unmanage,
  AppExtraFlag.force,
  AppExtraFlag.pinned,
];

export interface AppConfigurationExtended extends AppConfig {
  key: number;
}
