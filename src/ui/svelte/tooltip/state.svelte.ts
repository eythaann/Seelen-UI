import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";

let showing = $state<boolean>(false);
let text = $state<string | null>(null);

Widget.self.onTrigger(async ({ desiredPosition, alignX, alignY, customArgs }) => {
  const show = Boolean(customArgs?.show);
  if (!show) {
    showing = false;
    return;
  }

  if (desiredPosition) {
    await Widget.self.adjustAndSetPosition(desiredPosition.x, desiredPosition.y, alignX, alignY);
  }

  invoke(SeelenCommand.BringSelfToTop);
  text = String(customArgs?.text ?? "") || null;
  showing = true;
});

class State {
  get text() {
    return text;
  }

  get showing() {
    return showing;
  }
}

export const state = new State();
