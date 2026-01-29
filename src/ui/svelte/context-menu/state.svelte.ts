import { Widget } from "@seelen-ui/lib";
import type { ContextMenu } from "@seelen-ui/lib/types";

let data = $state<ContextMenu | null>(null);
let owner = $state<string | null>(null);
let forwardTo = $state<string | null>(null);

Widget.self.onTrigger(({ customArgs }) => {
  data = (customArgs?.menu as any) || null;
  owner = (customArgs?.owner as any) || null;
  forwardTo = (customArgs?.forwardTo as any) || null;
});

class State {
  get data() {
    return data;
  }

  get owner() {
    return owner;
  }

  get forwardTo() {
    return forwardTo;
  }
}

export const state = new State();
