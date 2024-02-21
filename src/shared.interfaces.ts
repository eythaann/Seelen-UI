import { StaticConfig } from './JsonSettings.interface';
import { ApplicationConfiguration } from './YamlSettings.interface';

export interface BackgroundApi {
  enableAutostart: () => void;
  disableAutostart: () => void;
  autostartTaskExist: () => Promise<boolean>;
  getUserSettings: (route?: string) => Promise<UserSettings>;
  saveUserSettings: (settings: UserSettings) => Promise<void>;
  loadAppsTemplate: () => Promise<ApplicationConfiguration[]>;
}

export interface UserSettings {
  jsonSettings: StaticConfig;
  yamlSettings: ApplicationConfiguration[];
}