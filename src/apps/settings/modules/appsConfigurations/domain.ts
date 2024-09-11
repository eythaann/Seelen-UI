import { AppConfiguration, AppExtraFlag } from 'seelen-core';

export enum WmApplicationOptions {
  Float = `${AppExtraFlag.Float}`,
  Unmanage = `${AppExtraFlag.Unmanage}`,
  ForceManage = `${AppExtraFlag.Force}`,
  Pinned = `${AppExtraFlag.Pinned}`,
}

export enum WegApplicationOptions {
  Hidden = `${AppExtraFlag.Hidden}`,
}

console.log(WmApplicationOptions, WegApplicationOptions);
export interface AppConfigurationExtended extends AppConfiguration {
  key: number;
}
