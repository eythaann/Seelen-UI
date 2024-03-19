import { StaticConfig } from './JsonSettings.interface';
import { ApplicationConfiguration } from './YamlSettings.interface';

export interface UserSettings {
  jsonSettings: StaticConfig;
  yamlSettings: ApplicationConfiguration[];
  ahkEnabled: boolean;
  updateNotification: boolean;
}

export interface AppTemplate {
  name: string;
  description: string;
  apps: ApplicationConfiguration[];
}