import { StaticConfig } from './JsonSettings.interface';
import { ApplicationConfiguration } from './YamlSettings.interface';

export interface BackgroundApi {
  autostartTaskExist: () => Promise<boolean>;
  // actions
  enableAutostart: () => void;
  disableAutostart: () => void;
  forceRestart: () => void;
  quit: () => void;
  // settings
  getUserSettings: (route?: string) => Promise<UserSettings>;
  saveUserSettings: (settings: UserSettings) => Promise<void>;
  importApps: () => Promise<ApplicationConfiguration[]>;
  loadAppsTemplates: () => Promise<AppTemplate[]>;
  exportApps: (apps: ApplicationConfiguration[]) => Promise<void>;
  // installers
  runAhkSetup: () => void;
}

export interface UserSettings {
  jsonSettings: StaticConfig;
  yamlSettings: ApplicationConfiguration[];
  ahkEnabled: boolean;
}

export interface AppTemplate {
  name: string;
  description: string;
  apps: ApplicationConfiguration[];
}