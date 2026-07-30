<script lang="ts">
  import type { StartMenuItem } from "@seelen-ui/lib/types";
  import { Widget, invoke, SeelenCommand } from "@seelen-ui/lib";
  import { FileIcon } from "libs/ui/svelte/components/Icon";
  import { t } from "../i18n";
  import type { createSortable } from "@dnd-kit/svelte/sortable";
  import { getItemContextMenu } from "./context-menu.svelte";
  import type { SelectionScope } from "../keyboard-navigation";

  interface Props {
    item: StartMenuItem;
    idx: number;
    isActiveDropzone?: boolean;
    lazy?: boolean;
    sortable?: ReturnType<typeof createSortable> | null;
    scope: SelectionScope;
  }

  let {
    item,
    idx,
    isActiveDropzone = false,
    lazy = false,
    sortable = null,
    scope,
  }: Props = $props();

  let el: HTMLLIElement;

  const itemId = $derived(item.umid || item.path.toLowerCase());
  const isSelected = $derived(scope.getSelected() === itemId);
  const tabindex = $derived(isSelected || (idx === 0 && !scope.getSelected()) ? 0 : -1);

  const menu = $derived(getItemContextMenu(item, $t));
  const noopAttach = () => {};

  $effect(() => {
    if (isSelected) {
      el.scrollIntoView({ block: "nearest" });
    }
  });

  function handleClick() {
    Widget.self.hide();
    const program = item.umid ? `shell:AppsFolder\\${item.umid}` : item.path;
    invoke(SeelenCommand.OpenFile, { path: program });
  }

  function handleContextMenu() {
    invoke(SeelenCommand.TriggerContextMenu, {
      menu,
      forwardTo: null,
    });
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
  {@attach sortable?.attach ?? noopAttach}
  role="option"
  aria-selected={isSelected}
  {tabindex}
  data-item-id={itemId}
  class="app"
  class:is-dragging={sortable?.isDragging}
  class:is-dropping={sortable?.isDropping}
  class:is-drop-target={isActiveDropzone}
  onclick={handleClick}
  oncontextmenu={handleContextMenu}
  onkeydown={handleKeyDown}
>
  <FileIcon class="app-icon" path={item.path} umid={item.umid} {lazy} />
  <div class="app-name" title={item.display_name}>
    {item.display_name}
  </div>
</li>
