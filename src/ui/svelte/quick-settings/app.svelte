<script lang="ts">
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import type { WidgetId } from "@seelen-ui/lib/types";
  import { Icon } from "libs/ui/svelte/components/Icon";
  import BrightnessControl from "./components/BrightnessControl.svelte";
  import MediaDevices from "./components/MediaDevices.svelte";
  import RadioButtons from "./components/RadioButtons.svelte";

  function openAppSettings() {
    invoke(SeelenCommand.TriggerWidget, {
      payload: { id: "@seelen/settings" as WidgetId },
    });
  }

  function openPowerMenu() {
    invoke(SeelenCommand.TriggerWidget, {
      payload: { id: "@seelen/power-menu" as WidgetId },
    });
  }

  $effect(() => {
    Widget.getCurrent().ready();
  });
</script>

<div class={["slu-standard-popover", "quick-settings"]}>
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
