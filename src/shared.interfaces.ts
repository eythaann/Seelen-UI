import { StaticConfig } from './JsonSettings.interface';

export interface BackgroundApi {
  enableAutostart: () => void;
  disableAutostart: () => void;
  autostartTaskExist: () => Promise<boolean>;
  getUserSettings: () => Promise<UserSettings>;
}

export interface UserSettings {
  jsonSettings: StaticConfig | null;
  yamlSettings: any[] | null;
}