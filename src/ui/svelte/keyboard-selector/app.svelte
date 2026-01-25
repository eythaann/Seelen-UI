<script lang="ts">
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import { state } from "./state.svelte";
  import { t } from "./i18n";

  $effect(() => {
    Widget.getCurrent().ready();
  });

  function onKeyboardClick(id: string, handle: string) {
    invoke(SeelenCommand.SystemSetKeyboardLayout, {
      id,
      handle,
    });
  }

  function openKeyboardSettings() {
    invoke(SeelenCommand.OpenFile, { path: "ms-settings:keyboard" });
  }
</script>

<div class={["slu-standard-popover", "keyboard-selector"]}>
  <div class="keyboard-selector-header">{$t("title")}</div>
  <div class="keyboard-selector-body">
    {#each state.langs as lang}
      {#each lang.keyboardLayouts as keyboard (keyboard.handle)}
        <button
          class="layout"
          class:layout-active={keyboard.active}
          onclick={() => onKeyboardClick(keyboard.id, keyboard.handle)}
        >
          <div class="layout-icon">
            <Icon iconName="FaRegKeyboard" />
          </div>
          <div class="layout-info">
            <span class="layout-lang">
              {lang.name}
            </span>
            <span class="layout-keyboard">
              {keyboard.displayName}
            </span>
          </div>
        </button>
      {/each}
    {/each}
  </div>
  <div class="keyboard-selector-footer">
    <button data-skin="transparent" onclick={openKeyboardSettings}>
      {$t("more")}
    </button>
  </div>
</div>
