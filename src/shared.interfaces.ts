import { StaticConfig } from './JsonSettings.interface';
import { ApplicationConfiguration } from './YamlSettings.interface';

export interface BackgroundApi {
  enableAutostart: () => void;
  disableAutostart: () => void;
  autostartTaskExist: () => Promise<boolean>;
  getUserSettings: () => Promise<UserSettings>;
  saveUserSettings: (settings: UserSettings) => Promise<void>;
}

export interface UserSettings {
  jsonSettings: StaticConfig;
  yamlSettings: ApplicationConfiguration[];
}