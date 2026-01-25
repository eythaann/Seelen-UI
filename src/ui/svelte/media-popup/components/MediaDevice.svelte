<script lang="ts">
  import type { MediaDevice } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import { t } from "../i18n";

  interface Props {
    device: MediaDevice;
    setViewDeviceId: (id: string) => void;
  }

  let { device, setViewDeviceId }: Props = $props();

  async function onClickMultimedia() {
    if (!device.isDefaultMultimedia) {
      try {
        await invoke(SeelenCommand.MediaSetDefaultDevice, {
          id: device.id,
          role: "multimedia",
        });
        await invoke(SeelenCommand.MediaSetDefaultDevice, {
          id: device.id,
          role: "console",
        });
      } catch (e) {
        console.error(e);
      }
    }
  }

  async function onClickCommunications() {
    if (!device.isDefaultCommunications) {
      try {
        await invoke(SeelenCommand.MediaSetDefaultDevice, {
          id: device.id,
          role: "communications",
        });
      } catch (e) {
        console.error(e);
      }
    }
  }
</script>

<div class="media-device">
  <div data-behavior="group">
    <button
      data-skin={device.isDefaultMultimedia ? "solid" : "default"}
      onclick={onClickMultimedia}
      title={$t("device.multimedia")}
    >
      <Icon iconName="IoMusicalNotes" size={14} />
    </button>
    <button
      data-skin={device.isDefaultCommunications ? "solid" : "default"}
      onclick={onClickCommunications}
      title={$t("device.comunications")}
    >
      <Icon iconName="FaPhoneFlip" size={12} />
    </button>
  </div>
  <span class="media-device-name">{device.name}</span>
  <button data-skin="transparent" onclick={() => setViewDeviceId(device.id)}>
    <Icon iconName="RiEqualizerLine" />
  </button>
</div>
