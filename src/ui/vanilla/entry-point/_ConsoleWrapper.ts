// this file is a modification of https://github.com/tauri-apps/tauri-plugin-log/blob/v2/guest-js/index.ts
import { _invoke, webviewInfo } from "./_tauri.ts";

export interface LogOptions {
  file?: string;
  line?: number;
  keyValues?: Record<string, string | undefined>;
}

export enum LogLevel {
  Trace = 1,
  Debug,
  Info,
  Warn,
  Error,
}

async function log(
  level: LogLevel,
  message: string,
  _options?: LogOptions,
): Promise<void> {
  // we use the webview label as the location, instead of call stack as the stack on the Seelen UI case
  // will be always the same because of the console wrapper
  const location = webviewInfo.label;
  await _invoke("log_from_webview", { level, message, location });
}

export async function error(
  message: string,
  options?: LogOptions,
): Promise<void> {
  await log(LogLevel.Error, message, options);
}

export async function warn(
  message: string,
  options?: LogOptions,
): Promise<void> {
  await log(LogLevel.Warn, message, options);
}

export async function info(
  message: string,
  options?: LogOptions,
): Promise<void> {
  await log(LogLevel.Info, message, options);
}

export async function debug(
  message: string,
  options?: LogOptions,
): Promise<void> {
  await log(LogLevel.Debug, message, options);
}

export async function trace(
  message: string,
  options?: LogOptions,
): Promise<void> {
  await log(LogLevel.Trace, message, options);
}
