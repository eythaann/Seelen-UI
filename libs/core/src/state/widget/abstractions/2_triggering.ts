import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { WidgetTriggerPayload } from "@seelen-ui/types";
import { SeelenEvent } from "../../../handlers/mod.ts";

import { Widget_1 } from "./1_rect.ts";

type unsubscriber = () => void;

let _triggerSubs: Set<(args: WidgetTriggerPayload) => void>;
function registerTrigger(cb: (args: WidgetTriggerPayload) => void): unsubscriber {
  if (!_triggerSubs) {
    _triggerSubs = new Set();
    getCurrentWebview().listen<WidgetTriggerPayload>(SeelenEvent.WidgetTriggered, ({ payload }) => {
      _triggerSubs.forEach((cb) => cb(payload));
    });
  }
  _triggerSubs.add(cb);
  return () => {
    _triggerSubs.delete(cb);
  };
}

export abstract class Widget_2 extends Widget_1 {
  public onTrigger(cb: (args: WidgetTriggerPayload) => void): unsubscriber {
    return registerTrigger(cb);
  }
}
