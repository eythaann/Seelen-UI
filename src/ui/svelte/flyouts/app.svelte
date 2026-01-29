<script lang="ts">
  import { Widget, invoke, SeelenCommand } from "@seelen-ui/lib";
  import { state as gState } from "./state/mod.svelte";
  import { Icon, SpecificIcon } from "libs/ui/svelte/components/Icon";
  import { Monitors, setShowing } from "./state/placement.svelte";
  import { brightnessIcon, nanosecondsToPlayingTime, outputVolumeIcon } from "libs/ui/utils";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { debounce, throttle } from "lodash";
  import { ConfigState } from "./state/config.svelte";

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

  let playing = $derived.by(() => {
    return gState.mediaPlaying.find((p) => p.default);
  });

  let vd = $derived.by(() => {
    if (Monitors.primary) {
      return gState.workspaces.monitors[Monitors.primary.id] || null;
    }
    return null;
  });

  let activeWorkspaceData = $derived.by(() => {
    return vd?.workspaces
      .map((workspace, idx) => ({ ...workspace, idx }))
      .find((workspace) => workspace.id == vd.active_workspace);
  });

  let volume = $derived(output?.volume || 0);
  let playingTitle = $derived(playing?.title);
  let brightnessLevel = $derived(gState.brightness?.current || 0);
  let activeWorkspace = $derived(vd?.active_workspace);

  // svelte-ignore state_referenced_locally
  const prev = {
    volume,
    playingTitle,
    brightnessLevel,
    activeWorkspace,
  };

  const hideWithDelay = $derived(
    debounce(() => {
      setShowing(false);
    }, ConfigState.config.timeToShow * 1000),
  );

  $effect(() => {
    let somethingChanged = false;

    if (prev.volume.toFixed(2) !== volume.toFixed(2)) {
      prev.volume = volume;
      lastChanged = "mediaDevices";
      somethingChanged = true;
    }

    if (prev.playingTitle !== playingTitle) {
      prev.playingTitle = playingTitle;
      lastChanged = "mediaPlaying";
      somethingChanged = true;
    }

    if (prev.brightnessLevel !== brightnessLevel) {
      prev.brightnessLevel = brightnessLevel;
      lastChanged = "brightness";
      somethingChanged = true;
    }

    if (prev.activeWorkspace !== activeWorkspace) {
      prev.activeWorkspace = activeWorkspace;
      lastChanged = "workspace";
      somethingChanged = true;
    }

    if (somethingChanged) {
      setShowing(true);
      hideWithDelay();
    }
  });

  const setBrightnessThrottled = throttle((brightness: number) => {
    invoke(SeelenCommand.SetMainMonitorBrightness, { brightness });
  }, 100);

  const setVolumeThrottled = throttle((deviceId: string, level: number) => {
    invoke(SeelenCommand.SetVolumeLevel, {
      deviceId,
      sessionId: null,
      level,
    });
  }, 100);

  function toggleMute(deviceId: string) {
    invoke(SeelenCommand.MediaToggleMute, { deviceId, sessionId: null });
  }
</script>

<div class="flyout" data-placement={ConfigState.config.placement}>
  {#if lastChanged === "workspace" && activeWorkspaceData}
    <div class="workspace">
      <span class="workspace-name">
        {activeWorkspaceData.name || `Workspace ${activeWorkspaceData.idx + 1}`}
      </span>
      <Icon iconName={(activeWorkspaceData.icon as any) || "PiMonitorBold"} />
    </div>
  {/if}

  {#if lastChanged === "mediaDevices" && output}
    <div class="volume">
      <Icon
        iconName={outputVolumeIcon(output.muted, output.volume)}
        onclick={() => toggleMute(output.id)}
      />
      <input
        type="range"
        data-skin="flat"
        data-orientation={orientation}
        value={output.volume}
        oninput={(e) => {
          setVolumeThrottled(output.id, Number(e.currentTarget.value));
        }}
        min={0}
        max={1}
        step={0.01}
      />
    </div>
  {/if}

  {#if lastChanged === "brightness" && gState.brightness}
    <div class="brightness">
      <Icon iconName={brightnessIcon(gState.brightness.current)} />
      <input
        type="range"
        data-skin="flat"
        data-orientation={orientation}
        value={gState.brightness.current}
        oninput={(e) => {
          setBrightnessThrottled(Number(e.currentTarget.value));
        }}
        min={gState.brightness.min}
        max={gState.brightness.max}
      />
    </div>
  {/if}

  {#if lastChanged === "mediaPlaying" && playing}
    <div class="player">
      <div class="player-thumbnail-container">
        {#if playing.thumbnail}
          <img src={convertFileSrc(playing.thumbnail)} alt="" />
        {:else}
          <SpecificIcon name="defaultPlayerThumbnail" />
        {/if}
      </div>

      <div class="player-info">
        <div class="player-title">{playing.title}</div>
        <div class="player-author">{playing.author}</div>
        <div class="player-timeline">
          <span>{nanosecondsToPlayingTime(playing.timeline.position as any)}</span>
          <span>/</span>
          <span>{nanosecondsToPlayingTime(playing.timeline.end as any)}</span>
        </div>
      </div>

      <div class="player-controls">
        <button
          data-skin="transparent"
          onclick={() => invoke(SeelenCommand.MediaPrev, { id: playing.umid })}
        >
          <Icon iconName="IoPlaySkipBack" />
        </button>
        <button
          data-skin="transparent"
          onclick={() => invoke(SeelenCommand.MediaTogglePlayPause, { id: playing.umid })}
        >
          <Icon iconName={playing.playing ? "IoPause" : "IoPlay"} />
        </button>
        <button
          data-skin="transparent"
          onclick={() => invoke(SeelenCommand.MediaNext, { id: playing.umid })}
        >
          <Icon iconName="IoPlaySkipForward" />
        </button>
      </div>

      <progress
        class="player-progress"
        value={playing.timeline.position as any}
        max={playing.timeline.end as any}
      >
        <span>{nanosecondsToPlayingTime(playing.timeline.position as any)}</span>
        <span>/</span>
        <span>{nanosecondsToPlayingTime(playing.timeline.end as any)}</span>
      </progress>
    </div>
  {/if}
</div>
