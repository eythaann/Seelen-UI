import { FileIcon } from "@shared/components/Icon";
import { useEffect, useRef } from "react";
import { $selectedWindow, $showing, $windows } from "./state.ts";
import type { UserAppWindow } from "@seelen-ui/lib/types";
import { invoke, SeelenCommand } from "@seelen-ui/lib";

interface Props {
  data: UserAppWindow;
  index: number;
}

export function Item({ data, index }: Props) {
  const buttonRef = useRef<HTMLButtonElement>(null);
  const isSelected = data.hwnd === $selectedWindow.value;

  useEffect(() => {
    if (isSelected && buttonRef.current) {
      buttonRef.current.focus();
    }
  }, [isSelected]);

  function handleKeyDown(e: React.KeyboardEvent<HTMLButtonElement>) {
    // Handle Enter key to activate the window
    if (e.key === "Enter") {
      e.preventDefault();
      buttonRef.current?.click();
      return;
    }

    // Handle navigation keys
    const isNavigationKey = e.key === "Tab" || e.key === "ArrowRight" || e.key === "ArrowLeft";

    if (isNavigationKey) {
      e.preventDefault();

      const direction = e.key === "ArrowLeft" || (e.key === "Tab" && e.shiftKey) ? "previous" : "next";

      navigateToItem(direction, index);
    }
  }

  return (
    <button
      ref={buttonRef}
      class="task"
      onKeyDown={handleKeyDown}
      onClick={() => {
        invoke(SeelenCommand.WegToggleWindowState, {
          hwnd: data.hwnd,
          wasFocused: false,
        });
        $showing.value = false;
      }}
      onFocus={() => {
        $selectedWindow.value = data.hwnd;
      }}
    >
      <div class="task-icon">
        <FileIcon umid={data.umid} path={data.process.path} />
      </div>
      <div class="task-title">{data.appName}</div>
    </button>
  );
}

// Navigation helper functions
function getNextIndex(currentIndex: number, totalItems: number): number {
  return (currentIndex + 1) % totalItems;
}

function getPreviousIndex(currentIndex: number, totalItems: number): number {
  return (currentIndex - 1 + totalItems) % totalItems;
}

function navigateToItem(direction: "next" | "previous", currentIndex: number): void {
  const windows = $windows.value;
  const totalItems = windows.length;

  if (totalItems === 0) return;

  const nextIndex = direction === "next"
    ? getNextIndex(currentIndex, totalItems)
    : getPreviousIndex(currentIndex, totalItems);

  $selectedWindow.value = windows[nextIndex]?.hwnd || null;
}
