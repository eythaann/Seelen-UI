import type Sandbox from "@nyariv/sandboxjs";
import { Alignment, SeelenWegSide, type WidgetId } from "@seelen-ui/lib/types";
import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { settingsState, widgetRect } from "./state/settings.svelte.ts";
import { evalSanboxed } from "libs/ui/svelte/utils/sandbox.ts";

const ALLOWED_COMMANDS: SeelenCommand[] = [
  SeelenCommand.OpenFile,
  SeelenCommand.ShowDesktop,
  SeelenCommand.ShowStartMenu,
];

const ActionsScope = {
  SeelenCommand,
  invoke(command: SeelenCommand, args?: any) {
    if (!ALLOWED_COMMANDS.includes(command)) {
      console.warn(`Trying to execute command that is not allowed: "${command}"`);
      return;
    }
    invoke(command, args);
  },
};

export function evalActionSanboxed(
  executor: ReturnType<Sandbox["compile"]> | null,
  scope: Record<string, any>,
): void {
  evalSanboxed(executor, { ...scope, ...ActionsScope });
}

export function stringFromEvaluated(content: unknown): string {
  switch (typeof content) {
    case "string":
      return content;
    case "number":
    case "boolean":
    case "bigint":
      return String(content);
    case "object":
      if (content === null) return "";
      if (Array.isArray(content)) return content.map(stringFromEvaluated).join("");
      return "";
    default:
      return "";
  }
}

export function triggerWidget(widgetId: WidgetId, itemId: string): void {
  if (typeof widgetId !== "string") {
    return;
  }

  const element = document.getElementById(itemId);
  if (!element) {
    console.error(`Element with id ${itemId} not found`);
    return;
  }

  const dockSide = settingsState.position;
  const elRect = element.getBoundingClientRect();
  const viewRect = widgetRect.value.hitboxRect;

  const toPhysical = (n: number) => Math.round(n * globalThis.devicePixelRatio);

  let x: number;
  let y: number;
  let alignX: Alignment;
  let alignY: Alignment;

  switch (dockSide) {
    case SeelenWegSide.Bottom:
      x = viewRect.left + toPhysical(elRect.left + elRect.width / 2);
      y = viewRect.top;
      alignX = Alignment.Center;
      alignY = Alignment.End;
      break;
    case SeelenWegSide.Top:
      x = viewRect.left + toPhysical(elRect.left + elRect.width / 2);
      y = viewRect.bottom;
      alignX = Alignment.Center;
      alignY = Alignment.Start;
      break;
    case SeelenWegSide.Left:
      x = viewRect.right;
      y = viewRect.top + toPhysical(elRect.top + elRect.height / 2);
      alignX = Alignment.Start;
      alignY = Alignment.Center;
      break;
    case SeelenWegSide.Right:
      x = viewRect.left;
      y = viewRect.top + toPhysical(elRect.top + elRect.height / 2);
      alignX = Alignment.End;
      alignY = Alignment.Center;
      break;
  }

  invoke(SeelenCommand.TriggerWidget, {
    payload: {
      id: widgetId,
      desiredPosition: { x, y },
      alignX,
      alignY,
    },
  });
}
