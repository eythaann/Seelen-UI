import { CSSProperties } from 'react';

export interface IRootState<T> {
  settings: T;
  theme: Theme;
}

export interface UserSettings {
  jsonSettings: anyObject;
  yamlSettings: anyObject[];
  ahkEnabled: boolean;
  themes: Theme[];
  theme: Theme | null;
}

export interface AppTemplate {
  name: string;
  description: string;
  apps: anyObject[];
}

export interface ThemeInfo {
  filename: string;
  cssFileUrl: string | null;
  displayName: string;
  author: string;
  description: string;
}

export interface Theme {
  info: ThemeInfo;
  variables: Record<`--${string}`, string>;
  seelenweg: {
    backgroundLayers: CSSProperties[] | number;
    items: {
      backgroundLayers: CSSProperties[] | number;
    };
    contextMenu: {
      backgroundLayers: CSSProperties[] | number;
    };
    preview: {
      backgroundLayers: CSSProperties[] | number;
    };
  };
}

export const defaultTheme: Theme = {
  info: {
    cssFileUrl: null,
    filename: 'unknown',
    displayName: 'Empty',
    author: 'unknown',
    description: 'unknown',
  },
  variables: {},
  seelenweg: {
    backgroundLayers: [],
    items: {
      backgroundLayers: [],
    },
    contextMenu: {
      backgroundLayers: [],
    },
    preview: {
      backgroundLayers: [],
    },
  },
};