import { IconPackManager } from "@seelen-ui/lib";
import type { Icon } from "@seelen-ui/lib/types";
import { prefersDarkColorScheme } from "../../runes/DarkMode.svelte";

const manager = await IconPackManager.create();

let innerState = $state.raw({
  _version: 0,
  value: manager,
});

manager.onChange(() => {
  // trick to make svelte re-render
  innerState = {
    _version: iconPackManager._version + 1,
    value: manager,
  };
});

export interface IconState {
  src: string | null;
  mask: string | null;
  isAproximatelySquare: boolean;
}

class IconPackManagerWrap {
  get value() {
    return innerState.value;
  }

  get _version() {
    return innerState._version;
  }

  resolve(icon: Icon): IconState {
    return {
      src: (prefersDarkColorScheme.value ? icon.dark : icon.light) || icon.base,
      mask: icon.mask,
      isAproximatelySquare: icon.isAproximatelySquare,
    };
  }
}

export const iconPackManager = new IconPackManagerWrap();
