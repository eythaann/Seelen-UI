import { StaticConfig } from './JsonSettings.interface';

export interface BackgroundApi {
  enableAutostart: () => void;
  disableAutostart: () => void;
  autostartTaskExist: () => Promise<boolean>;
  getUserSettings: () => Promise<UserSettings>;
  saveUserSettings: (settings: UserSettings) => Promise<void>;
}

export interface UserSettings {
  jsonSettings: StaticConfig;
  yamlSettings: any[];
}