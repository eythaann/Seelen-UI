<script lang="ts">
  import type { UserAppWindow } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { globalState } from "../state.svelte";
  import { FileIcon, Icon } from "libs/ui/svelte/components/Icon";
  import MissingIcon from "libs/ui/svelte/components/Icon/MissingIcon.svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";

  interface Props {
    task: UserAppWindow;
    index: number;
  }

  let { task, index }: Props = $props();

  let boxRef: HTMLDivElement | undefined = $state();
  const isSelected = $derived(task.hwnd === globalState.selectedWindow);
  const preview = $derived(globalState.previews[task.hwnd]);

  // Focus button when selected
  $effect(() => {
    if (isSelected && boxRef) {
      boxRef.focus();
    }
  });

  function handleKeyDown(e: KeyboardEvent) {
    // Handle Enter key to activate the window
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      boxRef?.click();
      return;
    }

    // Handle navigation keys
    const isNavigationKey = e.key === "Tab" || e.key === "ArrowRight" || e.key === "ArrowLeft";

    if (isNavigationKey) {
      e.preventDefault();

      const direction =
        e.key === "ArrowLeft" || (e.key === "Tab" && e.shiftKey) ? "previous" : "next";

      navigateToItem(direction, index);
    }
  }

  function handleClick() {
    globalState.showing = false;
    invoke(SeelenCommand.WegToggleWindowState, {
      hwnd: task.hwnd,
      wasFocused: false,
    });
    // Optimistically reorder UI before backend updates
    globalState.moveSelectedToFront(task.hwnd);
  }

  function handleFocus() {
    globalState.selectedWindow = task.hwnd;
  }

  // Navigation helper functions
  function getNextIndex(currentIndex: number, totalItems: number): number {
    return (currentIndex + 1) % totalItems;
  }

  function getPreviousIndex(currentIndex: number, totalItems: number): number {
    return (currentIndex - 1 + totalItems) % totalItems;
  }

  function navigateToItem(direction: "next" | "previous", currentIndex: number): void {
    const windows = globalState.windows;
    const totalItems = windows.length;

    if (totalItems === 0) return;

    const nextIndex =
      direction === "next"
        ? getNextIndex(currentIndex, totalItems)
        : getPreviousIndex(currentIndex, totalItems);

    globalState.selectedWindow = windows[nextIndex]?.hwnd || null;
  }
</script>

<div
  bind:this={boxRef}
  class="task"
  role="button"
  tabindex="0"
  onkeydown={handleKeyDown}
  onclick={handleClick}
  onfocus={handleFocus}
>
  <div class="task-header">
    <FileIcon class="task-icon" umid={task.umid} path={task.process.path} />
    <div class="task-title">{task.title}</div>
    <button
      data-skin="transparent"
      onclick={(e) => {
        e.stopPropagation();
        invoke(SeelenCommand.WegCloseApp, { hwnd: task.hwnd });
      }}
    >
      <Icon iconName="TbX" />
    </button>
  </div>
  <div class="task-preview-container">
    {#if preview}
      <img class="task-preview" src={convertFileSrc(preview.path) + "?v=" + preview.hash} alt="" />
    {:else}
      <MissingIcon class="task-no-preview" />
    {/if}
  </div>
</div>
