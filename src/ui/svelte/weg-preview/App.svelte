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

  /** Stable layout only needs a lease refresh + pointer check; DWM renders the frames itself. */
  const HEARTBEAT_MS = 100;

  let container: HTMLDivElement;
  let list: HTMLDivElement;
  let liveWindows = $state<number[]>([]);
  let previewChecked = $state(false);

  let hadVisible = false;
  let dismissed = false;
  let armed = false;

  // Layout is measured only when something invalidates it (scroll, resize, new cards, trigger).
  let layoutDirty = true;
  let candidates: LivePreviewFrame["targets"] = [];
  let viewport: PreviewRect = { left: 0, top: 0, right: 0, bottom: 0 };
  let frame = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;

  const live = new LivePreviewController({
    update: (session, frame) => invoke(SeelenCommand.UpdateWindowPreviews, { session, ...frame }),
    clear: (session) => invoke(SeelenCommand.ClearWindowPreviews, { session }),
    recover: (session, frame) => invoke(SeelenCommand.UpdateWindowPreviews, { session, ...frame, targets: [] }),
  }, onLiveStatus);

  function onLiveStatus(status: LivePreviewStatus, sent: LivePreviewFrame): void {
    if (status.dismissRequested) {
      void dismiss();
      return;
    }
    if (!status.visible) {
      if (hadVisible) {
        stop();
        live.clear();
      }
      return;
    }
    hadVisible = true;
    previewChecked = true;
    if (status.live.join() !== liveWindows.join()) liveWindows = status.live;
    if (sent.targets.some((target) => !status.live.includes(target.hwnd))) {
      previewState.enableSnapshotFallback();
    }
    if (!status.pointerInside && !dismissed) void dismiss();
  }

  function schedule(fast: boolean): void {
    if (!armed || frame) return;
    if (fast) {
      clearTimeout(timer);
      timer = undefined;
      frame = requestAnimationFrame(render);
    } else if (timer === undefined) {
      timer = setTimeout(() => {
        timer = undefined;
        render();
      }, HEARTBEAT_MS);
    }
  }

  function invalidate(): void {
    layoutDirty = true;
    schedule(true);
  }

  function stop(): void {
    armed = false;
    cancelAnimationFrame(frame);
    clearTimeout(timer);
    frame = 0;
    timer = undefined;
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
        hwnd: Number(element.dataset.previewHwnd),
        rect: { left: px(rect.left), top: px(rect.top), right: px(rect.right), bottom: px(rect.bottom) },
      };
    });
  }

  function render(): void {
    frame = 0;
    const anchor = previewState.sourceRect;
    if (!armed || !container || !anchor) return;

    const opacity = Math.round(Number(getComputedStyle(document.documentElement).opacity) * 255);
    if (!opacity && document.documentElement.hasAttribute("data-widget-hidden")) {
      schedule(false);
      return;
    }

    if (layoutDirty) measure();
    live.update({ targets: visiblePreviewTargets(candidates, viewport), opacity, anchor }, performance.now());

    // Follow show/hide fades frame by frame so the native surfaces fade with the popup.
    schedule(opacity < 255);
  }

  async function dismiss(): Promise<void> {
    if (dismissed) return;
    dismissed = true;
    await Widget.self.hide();
    // A new trigger can supersede the hide animation.
    if (dismissed) {
      stop();
      live.clear();
    }
  }

  function onClickPreview(hwnd: number): void {
    invoke(SeelenCommand.WegToggleWindowState, { hwnd, wasFocused: false });
    void dismiss();
  }

  function onClosePreview(e: MouseEvent, hwnd: number): void {
    e.stopPropagation();
    invoke(SeelenCommand.WegCloseApp, { hwnd });
  }

  function onAuxClickPreview(e: MouseEvent, hwnd: number): void {
    if (e.button === 1) invoke(SeelenCommand.WegCloseApp, { hwnd });
  }

  onMount(() => {
    const observer = new ResizeObserver(invalidate);
    observer.observe(container);
    observer.observe(list);
    return () => {
      observer.disconnect();
      stop();
      live.clear();
    };
  });

  // A persistent popup can be triggered for a different dock item without remounting.
  $effect(() => {
    if (!previewState.sourceRect) return;
    // Use the monitor's available height, not this autosized popup's own viewport height.
    container.style.setProperty(
      "--weg-preview-available-height",
      `${Math.max(160, Math.floor(screen.availHeight * 0.75))}px`,
    );
    live.clear();
    liveWindows = [];
    previewChecked = false;
    hadVisible = false;
    dismissed = false;
    armed = true;
    invalidate();
  });

  // Cards were added, removed or replaced: their rectangles moved.
  $effect(() => {
    if (previewState.currentInteractables.length === 0) {
      void dismiss();
      return;
    }
    invalidate();
  });

  $effect(() => {
    Widget.getCurrent().ready();
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
          {#if liveWindows.includes(win.hwnd)}
            <span class="weg-preview-live-surface" aria-hidden="true"></span>
          {:else if preview}
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
