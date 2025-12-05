<script lang="ts">
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import type { WidgetId } from "@seelen-ui/lib/types";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import BrightnessControl from "./components/BrightnessControl.svelte";
  import MediaDevices from "./components/MediaDevices.svelte";
  import RadioButtons from "./components/RadioButtons.svelte";

  function openAppSettings() {
    invoke(SeelenCommand.ShowAppSettings);
  }

  function openPowerMenu() {
    invoke(SeelenCommand.TriggerWidget, {
      payload: { id: "@seelen/power-menu" as WidgetId },
    });
  }
</script>

<div class={["slu-popover", "quick-settings"]}>
  <RadioButtons />
  <BrightnessControl />
  <MediaDevices />

  <div class="quick-settings-footer">
    <button data-skin="transparent" onclick={openAppSettings} title="App Settings">
      <Icon iconName="RiSettings4Fill" size={20} />
    </button>

    <button data-skin="transparent" onclick={openPowerMenu} title="Power">
      <Icon iconName="IoPower" size={20} />
    </button>
  </div>
</div>

<style>
  :global(body) {
    background-color: transparent;
    overflow: hidden;
  }

  :global(#root) {
    width: min-content;
    height: min-content;
  }
</style>
