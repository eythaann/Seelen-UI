<script lang="ts">
  import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
  import type { Rect as PreviewRect } from "@seelen-ui/lib/types";
  import { Icon } from "libs/ui/svelte/components/Icon/index.ts";
  import { onMount } from "svelte";
  import { t } from "./i18n/index.ts";
  import { previewState } from "./state.svelte.ts";
  import {
    contentViewport,
    type LivePreviewFrame,
    LivePreviewController,
    type LivePreviewStatus,
    visiblePreviewTargets,
  } from "./livePreview.ts";

  /** Surfaces are released if the hide fade did not reach opacity 0 by then (no animation). */
  const HIDE_RELEASE_MS = 250;

  const widget = Widget.getCurrent();

  let container: HTMLDivElement;
  let list: HTMLDivElement;
  let liveWindows = $state<string[]>([]);
  let previewChecked = $state(false);

  // Layout is measured only when something invalidates it (scroll, resize, new cards, trigger).
  let layoutDirty = true;
  let candidates: LivePreviewFrame["targets"] = [];
  let viewport: PreviewRect = { left: 0, top: 0, right: 0, bottom: 0 };
  let frame = 0;
  let releaseTimer: ReturnType<typeof setTimeout> | undefined;

  const live = new LivePreviewController({
    update: (session, frame) => invoke(SeelenCommand.UpdateWindowPreviews, { session, ...frame }),
    clear: (session) => invoke(SeelenCommand.ClearWindowPreviews, { session }),
  }, onLiveStatus);

  function onLiveStatus(status: LivePreviewStatus, sent: LivePreviewFrame): void {
    if (!status.visible) {
      // Shown by the lib but the native window is not visible yet: retry on the next frame.
      if (widget.visible) schedule();
      return;
    }
    previewChecked = true;
    if (status.live.join() !== liveWindows.join()) liveWindows = status.live;
    if (sent.targets.some((target) => !status.live.includes(target.hwnd))) {
      previewState.enableSnapshotFallback();
    }
  }

  function schedule(): void {
    if (!frame) frame = requestAnimationFrame(render);
  }

  function invalidate(): void {
    layoutDirty = true;
    if (widget.visible) schedule();
  }

  function release(): void {
    cancelAnimationFrame(frame);
    frame = 0;
    clearTimeout(releaseTimer);
    live.clear();
  }

  function measure(): void {
    layoutDirty = false;
    const px = (value: number) => Math.round(value * devicePixelRatio);
    const content = contentViewport(list.getBoundingClientRect(), list);
    viewport = {
      left: Math.ceil(content.left * devicePixelRatio),
      top: Math.ceil(content.top * devicePixelRatio),
      right: Math.floor(content.right * devicePixelRatio),
      bottom: Math.floor(content.bottom * devicePixelRatio),
    };
    candidates = Array.from(list.querySelectorAll<HTMLElement>("[data-preview-hwnd]"), (element) => {
      const rect = element.getBoundingClientRect();
      return {
        hwnd: element.dataset.previewHwnd!,
        rect: { left: px(rect.left), top: px(rect.top), right: px(rect.right), bottom: px(rect.bottom) },
      };
    });
  }

  function render(): void {
    frame = 0;
    if (!container) return;
    const opacity = Math.round(Number(getComputedStyle(document.documentElement).opacity) * 255);
    if (!widget.visible && opacity === 0) {
      release();
      return;
    }
    if (!previewState.liveEnabled) {
      // Performance mode: static snapshots only.
      release();
      liveWindows = [];
      previewChecked = true;
      previewState.enableSnapshotFallback();
      return;
    }
    if (layoutDirty) measure();
    live.update({ targets: visiblePreviewTargets(candidates, viewport), opacity });
    // Follow show/hide fades frame by frame so the native surfaces fade with the popup.
    if (opacity < 255) schedule();
  }

  function onClickPreview(hwnd: string): void {
    invoke(SeelenCommand.WegToggleWindowState, { hwnd, wasFocused: false });
    widget.hide();
  }

  function onClosePreview(e: MouseEvent, hwnd: string): void {
    e.stopPropagation();
    invoke(SeelenCommand.WegCloseApp, { hwnd });
  }

  function onAuxClickPreview(e: MouseEvent, hwnd: string): void {
    if (e.button === 1) invoke(SeelenCommand.WegCloseApp, { hwnd });
  }

  onMount(() => {
    const observer = new ResizeObserver(invalidate);
    observer.observe(container);
    observer.observe(list);
    const unsubscribe = widget.onVisibilityChange((visible) => {
      clearTimeout(releaseTimer);
      if (visible) {
        invalidate();
        return;
      }
      schedule();
      releaseTimer = setTimeout(() => {
        if (!widget.visible) release();
      }, HIDE_RELEASE_MS);
    });
    return () => {
      observer.disconnect();
      unsubscribe();
      release();
    };
  });

  // A persistent popup can be triggered for a different dock item without remounting.
  $effect(() => {
    if (!previewState.triggerCount) return;
    // Use the monitor's available height, not this autosized popup's own viewport height.
    container.style.setProperty(
      "--weg-preview-available-height",
      `${Math.max(160, Math.floor(screen.availHeight * 0.75))}px`,
    );
    live.clear();
    liveWindows = [];
    previewChecked = false;
    invalidate();
  });

  // Performance mode changed while shown: switch between live surfaces and snapshots.
  $effect(() => {
    void previewState.liveEnabled;
    invalidate();
  });

  // Cards were added, removed or replaced: their rectangles moved.
  $effect(() => {
    if (previewState.currentInteractables.length === 0) {
      widget.hide();
      return;
    }
    invalidate();
  });

  $effect(() => {
    widget.ready();
  });
</script>

<div bind:this={container} class="weg-item-preview-container slu-std-popover">
  <div bind:this={list} class="weg-item-preview-list" onscroll={invalidate}>
    {#each previewState.currentInteractables as win (win.hwnd)}
      {@const preview = previewState.previews.value[win.hwnd]}
      <div class="weg-item-preview">
        <div class="weg-item-preview-topbar">
          <div class="weg-item-preview-title">{win.title}</div>
          <button
            data-skin="transparent"
            aria-label={$t("preview.close_window")}
            onclick={(e) => onClosePreview(e, win.hwnd)}
          >
            <Icon iconName="IoClose" />
          </button>
        </div>
        <button
          data-skin="transparent"
          class="weg-item-preview-image-container"
          data-preview-hwnd={win.hwnd}
          aria-label={win.title}
          onclick={() => onClickPreview(win.hwnd)}
          onauxclick={(e) => onAuxClickPreview(e, win.hwnd)}
        >
          <!-- Live windows are composed natively by DWM over this area, which stays empty. -->
          {#if liveWindows.includes(win.hwnd)}{:else if preview}
            <img class="weg-item-preview-image" src="data:image/webp;base64,{preview.data}" alt={win.title} />
          {:else}
            <div class="weg-preview-placeholder">
              <Icon iconName="MdOutlinePreview" />
              <span>{$t(previewChecked ? "preview.unavailable" : "preview.loading")}</span>
            </div>
          {/if}
        </button>
      </div>
    {/each}
  </div>
</div>
