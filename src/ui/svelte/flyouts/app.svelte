<script lang="ts">
  import { Widget } from "@seelen-ui/lib";
  import { state as gState } from "./state/mod.svelte";
  import { RendererState, setShowing } from "./state/placement.svelte";
  import { ConfigState } from "./state/config.svelte";
  import { debounce } from "lodash";
  import Notification from "../notifications/components/Notification.svelte";
  import Workspace from "./app/Workspace.svelte";
  import MediaDevices from "./app/MediaDevices.svelte";
  import MediaPlaying from "./app/MediaPlaying.svelte";
  import Brightness from "./app/Brightness.svelte";

  $effect(() => {
    Widget.getCurrent().ready();
  });

  let lastChanged = $state<string | null>(null);
  let orientation = $derived(
    ["left", "right"].includes(ConfigState.config.placement) ? "vertical" : "horizontal",
  );

  let output = $derived.by(() => {
    return gState.mediaOutputs.find((o) => o.isDefaultMultimedia);
  });

  let recomendedPlayer = $derived.by(() => {
    return gState.mediaPlaying.find((p) => p.default);
  });

  let vd = $derived.by(() => {
    if (RendererState.primary) {
      return gState.workspaces.monitors[RendererState.primary.id] || null;
    }
    return null;
  });

  let activeWorkspaceData = $derived.by(() => {
    return vd?.workspaces
      .map((workspace, idx) => ({ ...workspace, idx }))
      .find((workspace) => workspace.id == vd.active_workspace);
  });

  // Use [0] (newest) instead of .pop() (oldest) on a descending-sorted array.
  let notification = $derived.by(
    () => gState.notifications.toSorted((a, b) => Number(b.date - a.date))[0],
  );

  let notificationId = $derived(notification?.id);
  let volume = $derived(output?.volume || 0);
  let playingTitle = $derived(recomendedPlayer?.title);
  let brightnessLevel = $derived(gState.brightness?.currentBrightness);
  let activeWorkspace = $derived(vd?.active_workspace);

  // svelte-ignore state_referenced_locally
  const prev = {
    volume,
    playingTitle,
    brightnessLevel,
    activeWorkspace,
    notificationId,
  };

  const hideWithDelay = $derived(
    debounce(() => {
      setShowing(false);
    }, ConfigState.config.timeToShow * 1000),
  );

  $effect(() => {
    let somethingChanged = false;

    // Guard with `output` so we never show an empty flyout when the device is removed.
    if (
      ConfigState.config.showVolumeChange &&
      prev.volume.toFixed(2) !== volume.toFixed(2) &&
      output
    ) {
      lastChanged = "mediaDevices";
      somethingChanged = true;
    }

    // Guard with `playing` so we never show an empty flyout when the player is closed.
    if (
      ConfigState.config.showMediaPlayerChange &&
      prev.playingTitle !== playingTitle &&
      recomendedPlayer
    ) {
      lastChanged = "mediaPlaying";
      somethingChanged = true;
    }

    // Guard with `gState.brightness` so we never show an empty flyout when brightness is unavailable.
    if (
      ConfigState.config.showBrightnessChange &&
      prev.brightnessLevel !== brightnessLevel &&
      gState.brightness
    ) {
      lastChanged = "brightness";
      somethingChanged = true;
    }

    // Guard with `activeWorkspaceData` so we never show an empty flyout on workspace edge cases.
    if (
      ConfigState.config.showWorkspaceChange &&
      prev.activeWorkspace !== activeWorkspace &&
      activeWorkspaceData
    ) {
      lastChanged = "workspace";
      somethingChanged = true;
    }

    // Guard with `notification` so we never show an empty flyout when the list becomes empty.
    if (
      ConfigState.config.showNotifications &&
      prev.notificationId !== notificationId &&
      notification
    ) {
      lastChanged = "notification";
      somethingChanged = true;
    }

    if (somethingChanged) {
      setShowing(true);
      hideWithDelay();
    }

    // Hide immediately when the currently displayed flyout loses its data source.
    if (
      (lastChanged === "mediaPlaying" && !recomendedPlayer) ||
      (lastChanged === "notification" && !notification)
    ) {
      hideWithDelay.cancel();
      setShowing(false);
      lastChanged = null;
    }

    prev.volume = volume;
    prev.playingTitle = playingTitle;
    prev.brightnessLevel = brightnessLevel;
    prev.activeWorkspace = activeWorkspace;
    prev.notificationId = notificationId;
  });
</script>

<div
  class={["slu-std-surface", "flyout"]}
  data-placement={ConfigState.config.placement}
  data-showing={RendererState.showing}
>
  {#if lastChanged === "notification" && notification}
    <Notification {notification} />
  {/if}

  {#if lastChanged === "workspace" && activeWorkspaceData}
    <Workspace workspace={activeWorkspaceData} />
  {/if}

  {#if lastChanged === "mediaDevices" && output}
    <MediaDevices {output} {orientation} />
  {/if}

  {#if lastChanged === "brightness" && gState.brightness}
    <Brightness brightness={gState.brightness} {orientation} />
  {/if}

  {#if lastChanged === "mediaPlaying" && recomendedPlayer}
    <MediaPlaying playing={recomendedPlayer} />
  {/if}
</div>

<style>
  .flyout[data-showing="false"] {
    opacity: 0;
  }
</style>
