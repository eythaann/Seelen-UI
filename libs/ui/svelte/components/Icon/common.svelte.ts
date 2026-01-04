import { IconPackManager } from "@seelen-ui/lib";

const manager = await IconPackManager.create();

export const iconPackManager = $state({
  _version: 0,
  value: manager,
});

manager.onChange(() => {
  // trick to make svelte re-render
  iconPackManager._version++;
});

export interface IconState {
  src: string | null;
  mask: string | null;
  isAproximatelySquare: boolean;
}
