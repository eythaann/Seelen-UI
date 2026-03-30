import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { SeelenSession } from "@seelen-ui/lib/types";
import { signal } from "@preact/signals";

export const session = signal<SeelenSession | null>(
  await invoke(SeelenCommand.GetSeelenSession),
);

subscribe(SeelenEvent.SeelenSessionChanged, ({ payload }) => {
  session.value = payload;
});
