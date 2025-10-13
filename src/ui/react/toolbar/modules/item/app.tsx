import Sandbox from "@nyariv/sandboxjs";
import { SeelenCommand, SeelenEvent } from "@seelen-ui/lib";
import { invoke } from "@tauri-apps/api/core";
import { emit, emitTo } from "@tauri-apps/api/event";

const ActionsScope = {
  open(path: string) {
    invoke(SeelenCommand.OpenFile, { path });
  },
  run(program: string, args: string[], workingDir: string) {
    invoke(SeelenCommand.Run, { program, args, workingDir });
  },
  copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
  },
  invoke(command: SeelenCommand, args?: any) {
    invoke(command, args);
  },
  emit(event: SeelenEvent, payload?: unknown) {
    emit(event, payload);
  },
  emitTo(target: string, event: SeelenEvent, payload?: unknown) {
    emitTo(target, event, payload);
  },
  SeelenCommand,
  SeelenEvent,
};

export async function EvaluateAction(code: string, scope: Record<string, any>) {
  const sandbox = new Sandbox();
  const executor = sandbox.compileAsync(code);
  await executor({ ...scope, ...ActionsScope }).run();
}
