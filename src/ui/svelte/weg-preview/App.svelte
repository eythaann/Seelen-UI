<script lang="ts">
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import { Icon } from "libs/ui/svelte/components/Icon/index.ts";
  import { previewState } from "./state.svelte.ts";
  import { t } from "libs/ui/svelte/utils/i18n.ts";
  import { onMount } from "svelte";
  import MediaControls from "./MediaControls.svelte";
  import { WindowPeekController } from "./windowPeek.ts";
  import { contentViewport, LivePreviewController, visiblePreviewTargets } from "./livePreview.ts";

  let failed = $state(false);
  let container: HTMLDivElement;
  let liveWindows = $state<number[]>([]);
  let previewChecked = $state(false);
  let liveVisible = false;
  let hadVisible = false;
  let dismissed = false;
  let armed = false;
  let wakeLive: () => void = () => {};
  let hovered: { hwnd: number; element: HTMLElement } | null = null;
  const peek = new WindowPeekController({
    begin: (hwnd, session, rect) => invoke(SeelenCommand.BeginWindowPeek, { hwnd, session, rect }),
    poll: (session, rect) => invoke(SeelenCommand.PollWindowPeek, { session, rect }),
    cancel: (session) => invoke(SeelenCommand.CancelWindowPeek, { session }),
    commit: (session) => invoke(SeelenCommand.CommitWindowPeek, { session }),
    activate: (hwnd) => invoke(SeelenCommand.WegToggleWindowState, { hwnd, wasFocused: false }),
  }, () => { void dismiss(); });
  const live = new LivePreviewController({
    update: (session, frame) => invoke(SeelenCommand.UpdateWindowPreviews, { session, ...frame }),
    clear: (session) => invoke(SeelenCommand.ClearWindowPreviews, { session }),
    recover: (session, frame) => invoke(SeelenCommand.UpdateWindowPreviews, { session, ...frame, targets: [] }),
  }, (status, frame) => {
    liveVisible = status.visible;
    if (!status.visible) {
      if (hadVisible) { armed = false; peek.cancel(); live.clear(); }
      return;
    }
    hadVisible = true;
    previewChecked = true;
    if (status.live.join() !== liveWindows.join()) liveWindows = status.live;
    if (frame.targets.some((window) => !status.live.includes(window.hwnd))) previewState.enableSnapshotFallback();
    if (!status.pointerInside && !peek.holding && !dismissed) void dismiss();
  });

  function hoverWindow(hwnd: number | null, element?: HTMLElement): void {
    hovered = hwnd !== null && element ? { hwnd, element } : null;
    peek.hover(hwnd, performance.now());
  }

  async function dismiss(): Promise<void> {
    if (dismissed) return;
    dismissed = true;
    hovered = null;
    peek.cancel();
    await Widget.self.hide();
    // A new trigger can supersede the hide animation.
    if (dismissed) { live.clear(); armed = false; liveVisible = false; }
  }

  onMount(() => {
    let polling = false;
    let animation = 0;
    const render = (now: number) => {
      animation = 0;
      if (!armed) return;
      animation = requestAnimationFrame(render);
      if (!armed || !container || !previewState.sourceRect) return;
      const opacity = Math.round(Number(getComputedStyle(document.documentElement).opacity) * 255);
      if (!opacity && document.documentElement.hasAttribute("data-widget-hidden")) return;
      const px = (value: number) => Math.round(value * devicePixelRatio);
      // Use the monitor's available height, not this autosized popup's own viewport height.
      const availableHeight = `${Math.max(160, Math.floor(screen.availHeight * 0.75))}px`;
      if (container.style.getPropertyValue("--weg-preview-available-height") !== availableHeight) container.style.setProperty("--weg-preview-available-height", availableHeight);
      const list = container.querySelector<HTMLElement>(".weg-item-preview-list")!;
      const bounds = list.getBoundingClientRect();
      const listRect = contentViewport({ left: bounds.left, top: bounds.top, right: bounds.right, bottom: bounds.bottom }, list);
      const candidates = Array.from(container.querySelectorAll<HTMLElement>("[data-preview-hwnd]")).map((element) => {
        const rect = element.getBoundingClientRect();
        return { hwnd: Number(element.dataset.previewHwnd), rect: { left: px(rect.left), top: px(rect.top), right: px(rect.right), bottom: px(rect.bottom) } };
      });
      const clip = { left: Math.ceil(listRect.left * devicePixelRatio), top: Math.ceil(listRect.top * devicePixelRatio), right: Math.floor(listRect.right * devicePixelRatio), bottom: Math.floor(listRect.bottom * devicePixelRatio) };
      const targets = visiblePreviewTargets(candidates, clip, hovered?.hwnd ?? null);
      live.update({ targets, opacity, anchor: previewState.sourceRect }, now);
    };
    wakeLive = () => { if (!animation) animation = requestAnimationFrame(render); };
    if (armed) wakeLive();
    const timer = setInterval(async () => {
      if (polling || !armed || dismissed || !liveVisible) return;
      polling = true;
      try {
        const hoveredRect = hovered?.element.isConnected ? hovered.element.getBoundingClientRect() : null;
        const px = (value: number) => Math.round(value * devicePixelRatio);
        await peek.tick(hoveredRect ? { left: px(hoveredRect.left), top: px(hoveredRect.top), right: px(hoveredRect.right), bottom: px(hoveredRect.bottom) } : null, performance.now());
      } catch (error) {
        console.error("Unable to track preview pointer", error);
      } finally {
        polling = false;
      }
    }, 80);
    return () => { clearInterval(timer); cancelAnimationFrame(animation); wakeLive = () => {}; peek.cancel(); live.clear(); };
  });

  // A persistent popup can be triggered for a different dock item without remounting.
  $effect(() => {
    const anchor = previewState.sourceRect;
    if (anchor) {
      hovered = null;
      peek.cancel();
      live.clear();
      liveWindows = [];
      previewChecked = false;
      liveVisible = false;
      hadVisible = false;
      dismissed = false;
      armed = true;
      wakeLive();
    }
  });

  $effect(() => {
    Widget.getCurrent().ready();
  });

  $effect(() => {
    if (previewState.currentInteractables.length === 0) {
      void dismiss();
    }
  });

  async function onClickPreview(hwnd: number) {
    failed = false;
    try {
      if (!await peek.commit(hwnd)) failed = true;
    } catch (error) {
      console.error("Unable to activate preview window", error);
      failed = true;
    }
  }

  function onClosePreview(e: MouseEvent, hwnd: number) {
    e.stopPropagation();
    hoverWindow(null);
    peek.cancel();
    invoke(SeelenCommand.WegCloseApp, { hwnd });
  }

  function onAuxClickPreview(e: MouseEvent, hwnd: number) {
    if (e.button === 1) {
      hoverWindow(null);
      peek.cancel();
      invoke(SeelenCommand.WegCloseApp, { hwnd });
    }
  }
</script>

<svelte:window onkeydown={(e) => { if (e.key === "Escape") void dismiss(); }} />

<div bind:this={container} role="group" class="weg-item-preview-container slu-std-popover" data-dock-side={previewState.position} onpointerdown={() => Widget.self.focus()}>
  {#if failed}<span class="media-transport-error" role="status">{$t("media.window_failed")}</span>{/if}
  <div class="weg-item-preview-list">
    {#each previewState.currentInteractables as win (win.hwnd)}
      {@const preview = previewState.previews.value[win.hwnd]}
      <div
        class="weg-item-preview"
      >
        <div class="weg-item-preview-topbar">
          <div class="weg-item-preview-title">{win.title}</div>
          <button data-skin="transparent" aria-label={$t("media.close_window")} onclick={(e) => onClosePreview(e, win.hwnd)}>
            <Icon iconName="IoClose" />
          </button>
        </div>
        <button data-skin="transparent" class="weg-item-preview-image-container" data-preview-hwnd={win.hwnd} aria-label={win.title} onpointerenter={(e) => hoverWindow(win.hwnd, e.currentTarget)} onpointerleave={() => hoverWindow(null)} onpointerdown={(e) => e.stopPropagation()} onclick={() => onClickPreview(win.hwnd)} onauxclick={(e) => onAuxClickPreview(e, win.hwnd)}>
          {#if liveWindows.includes(win.hwnd)}
            <span class="weg-preview-live-surface" aria-hidden="true"></span>
          {:else if preview}
            <img
              class="weg-item-preview-image"
              src="data:image/webp;base64,{preview.data}"
              alt={win.title}
            />
          {:else}
            <div class="weg-preview-placeholder"><Icon iconName="MdOutlinePreview" /><span>{$t(previewChecked ? "media.preview_unavailable" : "media.preview_loading")}</span></div>
          {/if}
        </button>
        <MediaControls window={win} />
      </div>
    {/each}
  </div>
</div>
