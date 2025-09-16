import type { SeelenCommandArgument, SeelenCommandReturn, SeelenEventPayload } from "@seelen-ui/types";
import { invoke as tauriInvoke, type InvokeOptions } from "@tauri-apps/api/core";
import { type EventCallback, listen, type Options as ListenerOptions } from "@tauri-apps/api/event";

import type { SeelenCommand } from "./commands.ts";
import type { SeelenEvent } from "./events.ts";

type $keyof<Type> = [Type] extends [never] ? keyof Type
  : Type extends Type ? keyof Type
  : never;

type UnionToIntersection<Type> = {
  [Key in $keyof<Type>]: Extract<
    Type,
    {
      [key in Key]?: unknown;
    }
  >[Key];
};

type MapNullToVoid<Obj> = {
  [K in keyof Obj]: [Obj[K]] extends [null] ? void : Obj[K];
};

type MapNullToUndefined<Obj> = {
  [K in keyof Obj]: [Obj[K]] extends [null] ? undefined : Obj[K];
};

export type AllSeelenCommandArguments = MapNullToUndefined<
  UnionToIntersection<SeelenCommandArgument>
>;
export type AllSeelenCommandReturns = MapNullToVoid<
  UnionToIntersection<SeelenCommandReturn>
>;

export type AllSeelenEventPayloads = UnionToIntersection<SeelenEventPayload>;

/**
 * Will call to the background process
 * @args Command to be called
 * @args Command arguments
 * @return Result of the command
 */
export function invoke<T extends SeelenCommand>(
  ...args: [AllSeelenCommandArguments[T]] extends [undefined] ? [
      command: T,
      args?: undefined,
      options?: InvokeOptions,
    ]
    : [
      command: T,
      args: AllSeelenCommandArguments[T],
      options?: InvokeOptions,
    ]
): Promise<AllSeelenCommandReturns[T]> {
  const [command, commandArgs, options] = args;
  return tauriInvoke(command, commandArgs, options);
}

export type UnSubscriber = () => void;

export function subscribe<T extends SeelenEvent>(
  event: T,
  cb: EventCallback<AllSeelenEventPayloads[T]>,
  options?: ListenerOptions,
): Promise<UnSubscriber> {
  return listen(event, cb, options);
}

export * from "./events.ts";
export * from "./commands.ts";
