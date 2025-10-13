import { FileIcon } from "@shared/components/Icon";
import { useEffect, useRef } from "react";
import { $selectedWindow, $showing } from "./state.ts";
import type { UserAppWindow } from "@seelen-ui/lib/types";
import { invoke, SeelenCommand } from "@seelen-ui/lib";

interface Props {
  data: UserAppWindow;
}

export function Item({ data }: Props) {
  const buttonRef = useRef<HTMLButtonElement>(null);
  const isSelected = data.hwnd === $selectedWindow.value;

  useEffect(() => {
    if (isSelected && buttonRef.current) {
      buttonRef.current.focus();
    }
  }, [isSelected]);

  function handleKeyDown(e: React.KeyboardEvent<HTMLButtonElement>) {
    if (e.key === "Enter") {
      e.preventDefault();
      buttonRef.current?.click();
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
    >
      <div class="task-icon">
        <FileIcon umid={data.umid} path={data.process.path} />
      </div>
      <div class="task-title">{data.appName}</div>
    </button>
  );
}
