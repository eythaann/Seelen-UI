// this file is a modification of https://github.com/tauri-apps/tauri-plugin-log/blob/v2/guest-js/index.ts
import { _invoke, WebviewInformation } from "./_tauri";

export interface LogOptions {
  file?: string;
  line?: number;
  keyValues?: Record<string, string | undefined>;
}

export enum LogLevel {
  /**
   * The "trace" level.
   *
   * Designates very low priority, often extremely verbose, information.
   */
  Trace = 1,
  /**
   * The "debug" level.
   *
   * Designates lower priority information.
   */
  Debug,
  /**
   * The "info" level.
   *
   * Designates useful information.
   */
  Info,
  /**
   * The "warn" level.
   *
   * Designates hazardous situations.
   */
  Warn,
  /**
   * The "error" level.
   *
   * Designates very serious errors.
   */
  Error,
}

const webviewInfo = new WebviewInformation();
async function log(
  level: LogLevel,
  message: string,
  options?: LogOptions,
): Promise<void> {
  // we use the webview label as the location, intead call stack as the stack on the Seelen UI case
  // will be always the same because of the console wrapper, this is different from the tauri plugin
  const location = webviewInfo.label;

  const { file, line, keyValues } = options ?? {};

  await _invoke("plugin:log|log", {
    level,
    message,
    location,
    file,
    line,
    keyValues,
  });
}

/**
 * Logs a message at the error level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { error } from '@tauri-apps/plugin-log';
 *
 * const err_info = "No connection";
 * const port = 22;
 *
 * error(`Error: ${err_info} on port ${port}`);
 * ```
 */
export async function error(
  message: string,
  options?: LogOptions,
): Promise<void> {
  await log(LogLevel.Error, message, options);
}

/**
 * Logs a message at the warn level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { warn } from '@tauri-apps/plugin-log';
 *
 * const warn_description = "Invalid Input";
 *
 * warn(`Warning! {warn_description}!`);
 * ```
 */
export async function warn(
  message: string,
  options?: LogOptions,
): Promise<void> {
  await log(LogLevel.Warn, message, options);
}

/**
 * Logs a message at the info level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { info } from '@tauri-apps/plugin-log';
 *
 * const conn_info = { port: 40, speed: 3.20 };
 *
 * info(`Connected to port {conn_info.port} at {conn_info.speed} Mb/s`);
 * ```
 */
export async function info(
  message: string,
  options?: LogOptions,
): Promise<void> {
  await log(LogLevel.Info, message, options);
}

/**
 * Logs a message at the debug level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { debug } from '@tauri-apps/plugin-log';
 *
 * const pos = { x: 3.234, y: -1.223 };
 *
 * debug(`New position: x: {pos.x}, y: {pos.y}`);
 * ```
 */
export async function debug(
  message: string,
  options?: LogOptions,
): Promise<void> {
  await log(LogLevel.Debug, message, options);
}

/**
 * Logs a message at the trace level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { trace } from '@tauri-apps/plugin-log';
 *
 * let pos = { x: 3.234, y: -1.223 };
 *
 * trace(`Position is: x: {pos.x}, y: {pos.y}`);
 * ```
 */
export async function trace(
  message: string,
  options?: LogOptions,
): Promise<void> {
  await log(LogLevel.Trace, message, options);
}
