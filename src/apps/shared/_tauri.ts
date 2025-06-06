// this file is used to reduce bundle size on widget loader, to mantain the minimal bundle size possible
// this file can't use dependencies
import type { InvokeArgs, InvokeOptions } from '@tauri-apps/api/core';

export class WebviewInformation {
  _label: string | null = null;
  get label() {
    if (this._label) {
      return this._label;
    }

    const viewLabel = window.__TAURI_INTERNALS__?.metadata?.currentWebview?.label;
    this._label = viewLabel ? decodeUrlSafeBase64(viewLabel) : 'Unknown';
    return this._label;
  }

  get widgetId() {
    const [id, _] = this.label.split('?');
    if (!id) {
      throw new Error('Invalid widget id');
    }
    return id;
  }
}

function decodeUrlSafeBase64(base64Str: string) {
  let standardBase64 = base64Str.replace(/-/g, '+').replace(/_/g, '/');
  const padLength = (4 - (standardBase64.length % 4)) % 4;
  standardBase64 += '='.repeat(padLength);
  return atob(standardBase64);
}

export function _invoke<T>(cmd: string, args?: InvokeArgs, options?: InvokeOptions): Promise<T> {
  return window.__TAURI_INTERNALS__!.invoke(cmd, args, options);
}
