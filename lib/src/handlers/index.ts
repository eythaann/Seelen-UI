export * from './invokers';

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import { SeelenCommand } from './invokers';

export enum EventHandler {
  UIColors = 'colors-changed',
  Settings = 'settings-changed',
  WegItems = 'weg-items',
  Themes = 'themes',
  Placeholders = 'placeholders',
  Layouts = 'layouts',
  SettingsByApp = 'settings-by-app',
  History = 'history',
}

export function Obtainable<T>(invokeKey: SeelenCommand, eventKey: EventHandler) {
  return class {
    static async getAsync(): Promise<T> {
      return await invoke(invokeKey);
    }

    static async onChange(cb: (value: T) => void) {
      await listen<T>(eventKey, (event) => {
        cb(event.payload);
      });
    }
  };
}
