import { StaticConfig } from './JsonSettings.interface';
import { ApplicationConfiguration } from './YamlSettings.interface';
import { CSSProperties } from 'react';

export interface UserSettings {
  jsonSettings: StaticConfig;
  yamlSettings: ApplicationConfiguration[];
  ahkEnabled: boolean;
  updateNotification: boolean;
  themes: Theme[];
  theme: Theme | null;
}

export interface AppTemplate {
  name: string;
  description: string;
  apps: ApplicationConfiguration[];
}

export interface ThemeInfo {
  filename: string;
  displayName: string;
  author: string;
}

export interface Theme {
  info: ThemeInfo;
  seelenweg: {
    background: CSSProperties[];
    items: {
      background: CSSProperties[];
    };
    contextMenu: {
      background: CSSProperties[];
    };
  };
}