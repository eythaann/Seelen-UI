<script lang="ts">
  import { SystemSettingKind, type SystemSettingEntry } from "@seelen-ui/lib/types";
  import { Widget, invoke, SeelenCommand } from "@seelen-ui/lib";
  import { FileIcon } from "libs/ui/svelte/components/Icon";
  import { getResultKey } from "../state/search.svelte";
  import type { SelectionScope } from "../keyboard-navigation";

  const SETTINGS_APP_UMID =
    "windows.immersivecontrolpanel_cw5n1h2txyewy!microsoft.windows.immersivecontrolpanel";
  const CONTROL_PANEL_UMID = "Microsoft.Windows.ControlPanel";

  interface Props {
    item: SystemSettingEntry;
    idx: number;
    scope: SelectionScope;
  }

  let { item, idx, scope }: Props = $props();

  let el: HTMLLIElement;

  const itemId = $derived(getResultKey(item));
  const isSelected = $derived(scope.getSelected() === itemId);
  const tabindex = $derived(isSelected || (idx === 0 && !scope.getSelected()) ? 0 : -1);
  const tooltip = $derived(item.description ? `${item.title}\n${item.description}` : item.title);

  $effect(() => {
    if (isSelected) {
      el.scrollIntoView({ block: "nearest" });
    }
  });

  function handleClick() {
    Widget.self.hide();
    invoke(SeelenCommand.OpenFile, { path: item.target });
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === " ") {
      e.stopPropagation();
      handleClick();
    }
  }
</script>

<li
  bind:this={el}
  role="option"
  aria-selected={isSelected}
  aria-label={tooltip}
  {tabindex}
  data-item-id={itemId}
  data-setting-kind={item.kind}
  class="app"
  onclick={handleClick}
  onkeydown={handleKeyDown}
>
  {#if item.glyph}
    <span class="app-icon app-icon-glyph" style:font-family={`"${item.fontFamily}"`}>
      {item.glyph}
    </span>
  {:else}
    <FileIcon
      class="app-icon"
      umid={item.kind === SystemSettingKind.ControlPanel ? CONTROL_PANEL_UMID : SETTINGS_APP_UMID}
    />
  {/if}
  <div class="app-name" title={tooltip}>
    {item.title}
  </div>
</li>
