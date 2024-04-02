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
  variables: Record<`--${string}`, string>;
  seelenweg: {
    background: CSSProperties[];
    items: {
      background: CSSProperties[];
      icon: CSSProperties;
    };
    contextMenu: {
      background: CSSProperties[];
      content: CSSProperties;
    };
    preview: {
      background: CSSProperties[];
      content: CSSProperties;
      items: {
        content: CSSProperties;
        title: CSSProperties;
        image: CSSProperties;
      };
    };
  };
}

export const defaultTheme: Theme = {
  info: {
    filename: 'unknown',
    displayName: 'Empty',
    author: 'none',
  },
  variables: {},
  seelenweg: {
    background: [],
    items: {
      background: [],
      icon: {},
    },
    contextMenu: {
      background: [],
      content: {},
    },
    preview: {
      content: {},
      background: [],
      items: {
        content: {},
        title: {},
        image: {},
      },
    },
  },
};