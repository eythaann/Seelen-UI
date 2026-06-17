<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { SpecificIcon } from "libs/ui/svelte/components/Icon/index.ts";
  import { t } from "../../i18n/index.ts";
  import type { StartMenuWegItem } from "../../types.ts";
  import { settingsState, getDockContextMenuAlignment } from "../../state/settings.svelte.ts";
  import { getMenuForItem } from "../../generalMenu.ts";
  import { delayedFocused } from "../../state/windows.svelte.ts";

  interface Props {
    item: StartMenuWegItem;
  }

  let { item }: Props = $props();

  const startMenuExes = ["SearchHost.exe", "StartMenuExperienceHost.exe"];
  const isStartMenuOpen = $derived(
    startMenuExes.some((program) => (delayedFocused.value?.exe || "").endsWith(program)),
  );

  function onClick() {
    if (!isStartMenuOpen) {
      invoke(SeelenCommand.ShowStartMenu);
    }
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
  class="weg-item weg-item-start"
  onclick={onClick}
  oncontextmenu={onContextMenu}
  onkeypress={() => {}}
>
  <SpecificIcon class="weg-item-icon weg-item-start-icon" name="@seelen/weg::start-menu" />
</div>
