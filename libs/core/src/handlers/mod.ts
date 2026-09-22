import type { SeelenEventPayload, SluCmdArgumentMap, SluCmdReturnMap } from "@seelen-ui/types";
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { type EventCallback, listen, type Options as ListenerOptions } from "@tauri-apps/api/event";

import { SeelenCommand } from "./commands.ts";
import { SeelenEvent } from "./events.ts";

type $keyof<Type> = [Type] extends [never] ? keyof Type : Type extends Type ? keyof Type : never;

type UnionToIntersection<Type> = {
  [Key in $keyof<Type>]: Extract<
    Type,
    {
      [key in Key]?: unknown;
    }
  >[Key];
};

type EmptyObject = Record<symbol, never>;

export type AllSeelenCommandArguments = UnionToIntersection<SluCmdArgumentMap>;
export type AllSeelenCommandReturns = UnionToIntersection<SluCmdReturnMap>;

/**
 * Will call to the background process
 * @args Command to be called
 * @args Command arguments
 * @return Result of the command
 */
export function invoke<T extends SeelenCommand>(
  ...args: EmptyObject extends Required<AllSeelenCommandArguments[T]> ? [command: T]
    : [command: T, args: AllSeelenCommandArguments[T]]
): Promise<AllSeelenCommandReturns[T]> {
  const [command, commandArgs] = args;
  return tauriInvoke(command, commandArgs);
}

export type UnSubscriber = () => void;
export type AllSeelenEventPayloads = UnionToIntersection<SeelenEventPayload>;

export function subscribe<T extends SeelenEvent>(
  event: T,
  cb: EventCallback<AllSeelenEventPayloads[T]>,
  options?: ListenerOptions,
): Promise<UnSubscriber> {
  return listen(event, cb, options);
}

export { SeelenCommand, SeelenEvent };
