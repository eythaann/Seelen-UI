<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { SpecificIcon } from "libs/ui/svelte/components/Icon/index.ts";
  import { t } from "../../i18n/index.ts";
  import type { ShowDesktopWegItem } from "../../types.ts";
  import { settingsState, getDockContextMenuAlignment } from "../../state/settings.svelte.ts";
  import { getMenuForItem } from "../../generalMenu.ts";

  interface Props {
    item: ShowDesktopWegItem;
  }

  let { item }: Props = $props();

  function onClick() {
    invoke(SeelenCommand.ShowDesktop);
  }

  function onContextMenu(e: MouseEvent) {
    e.stopPropagation();
    const { alignX, alignY } = getDockContextMenuAlignment(settingsState.position);
    invoke(SeelenCommand.TriggerContextMenu, {
      menu: { ...getMenuForItem($t, item), alignX, alignY },
      forwardTo: null,
    });
  }
</script>

<div
  role="button"
  tabindex="0"
  class="weg-item weg-item-show-desktop"
  onclick={onClick}
  oncontextmenu={onContextMenu}
  onkeypress={() => {}}
>
  <SpecificIcon class="weg-item-icon" name="@seelen/weg::show-desktop" />
</div>
