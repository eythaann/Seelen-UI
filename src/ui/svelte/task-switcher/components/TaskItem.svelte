<script lang="ts">
  import type { UserAppWindow } from "@seelen-ui/lib/types";
  import { invoke, SeelenCommand } from "@seelen-ui/lib";
  import { globalState } from "../state.svelte";
  import { FileIcon } from "libs/ui/svelte/components/Icon";

  interface Props {
    data: UserAppWindow;
    index: number;
  }

  let { data, index }: Props = $props();

  let buttonRef: HTMLButtonElement | undefined = $state();
  let isSelected = $derived(data.hwnd === globalState.selectedWindow);

  // Focus button when selected
  $effect(() => {
    if (isSelected && buttonRef) {
      buttonRef.focus();
    }
  });

  function handleKeyDown(e: KeyboardEvent) {
    // Handle Enter key to activate the window
    if (e.key === "Enter") {
      e.preventDefault();
      buttonRef?.click();
      return;
    }

    // Handle navigation keys
    const isNavigationKey =
      e.key === "Tab" || e.key === "ArrowRight" || e.key === "ArrowLeft";

    if (isNavigationKey) {
      e.preventDefault();

      const direction =
        e.key === "ArrowLeft" || (e.key === "Tab" && e.shiftKey) ? "previous" : "next";

      navigateToItem(direction, index);
    }
  }

  function handleClick() {
    invoke(SeelenCommand.WegToggleWindowState, {
      hwnd: data.hwnd,
      wasFocused: false,
    });
    globalState.showing = false;
  }

  function handleFocus() {
    globalState.selectedWindow = data.hwnd;
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

<button
  bind:this={buttonRef}
  class="task"
  onkeydown={handleKeyDown}
  onclick={handleClick}
  onfocus={handleFocus}
>
  <div class="task-icon">
    <FileIcon umid={data.umid} path={data.process.path} />
  </div>
  <!-- <div class="task-title">{data.appName}</div> -->
</button>
