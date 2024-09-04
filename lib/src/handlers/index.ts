import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export enum InvokeHandler {
  GetUIColors = 'get_ui_colors',
}

export enum EventHandler {
  UIColors = 'colors',
}

export function Obtainable<T>(invokeKey: InvokeHandler, eventKey: EventHandler) {
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