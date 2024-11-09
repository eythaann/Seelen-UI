export * from './invokers';
export * from './events';

import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

import { SeelenEvent } from './events';
import { SeelenCommand } from './invokers';

export function Obtainable<T>(invokeKey: SeelenCommand, eventKey: SeelenEvent) {
  return class {
    static async getAsync(): Promise<T> {
      return await invoke(invokeKey);
    }

    static async onChange(cb: (value: T) => void): Promise<UnlistenFn> {
      return listen<T>(eventKey, (event) => {
        cb(event.payload);
      });
    }
  };
}
