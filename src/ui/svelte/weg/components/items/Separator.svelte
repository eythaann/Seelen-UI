<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { HARDCODED_SEPARATOR_LEFT, HARDCODED_SEPARATOR_RIGHT } from "../../state/items.svelte.ts";
  import { settingsState } from "../../state/settings.svelte.ts";
  import { getMenuForItem } from "../../generalMenu.ts";
  import { t } from "../../i18n/index.ts";
  import type { SeparatorWegItem } from "../../types.ts";

  interface Props {
    item: SeparatorWegItem;
  }

  let { item }: Props = $props();

  const isSeparator1 = $derived(item.id === HARDCODED_SEPARATOR_LEFT.id);
  const isSeparator2 = $derived(item.id === HARDCODED_SEPARATOR_RIGHT.id);

  function onContextMenu(e: MouseEvent) {
    if (isSeparator1 || isSeparator2) return;
    e.stopPropagation();
    const alignX = settingsState.popupAlignX;
    const alignY = settingsState.popupAlignY;
    invoke(SeelenCommand.TriggerContextMenu, {
      menu: { ...getMenuForItem($t, item), alignX, alignY },
      forwardTo: null,
    });
  }
</script>

<div
  role="menuitem"
  tabindex="0"
  class="weg-separator"
  class:visible={!isSeparator1 && !isSeparator2}
  oncontextmenu={onContextMenu}
  onkeypress={() => {}}
></div>

<style>
  .weg-separator {
    opacity: 0;
    position: relative;

    &.visible {
      opacity: 1;
    }
  }

  :global(.vertical) {
    .weg-separator {
      width: var(--config-item-size);

      &.visible::after {
        content: "";
        position: absolute;
        left: 0;
        right: 0;
        top: calc(var(--config-space-between-items) / -2);
        bottom: calc(var(--config-space-between-items) / -2);
      }
    }
  }

  :global(.horizontal) {
    .weg-separator {
      height: var(--config-item-size);

      &.visible::after {
        content: "";
        position: absolute;
        top: 0;
        bottom: 0;
        left: calc(var(--config-space-between-items) / -2);
        right: calc(var(--config-space-between-items) / -2);
      }
    }
  }
</style>
