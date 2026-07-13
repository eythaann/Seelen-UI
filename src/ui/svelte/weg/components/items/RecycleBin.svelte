<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { SpecificIcon } from "libs/ui/svelte/components/Icon/index.ts";
  import { t } from "../../i18n/index.ts";
  import type { TrashBinItem } from "../../types.ts";
  import { settingsState } from "../../state/settings.svelte.ts";
  import { getMenuForItem } from "../../generalMenu.ts";
  import { trashBinInfo } from "../../state/system.svelte.ts";

  interface Props {
    item: TrashBinItem;
  }

  let { item }: Props = $props();

  const iconName = $derived(trashBinInfo.value.itemCount > 0 ? "bin::full" : "bin::empty");

  function onClick() {
    invoke(SeelenCommand.OpenFile, { path: "shell:RecycleBinFolder" });
  }

  function onContextMenu(e: MouseEvent) {
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
  role="button"
  tabindex="0"
  class="weg-item weg-item-show-desktop"
  onclick={onClick}
  oncontextmenu={onContextMenu}
  onkeypress={() => {}}
>
  <SpecificIcon class="weg-item-icon" name={iconName} />
</div>
