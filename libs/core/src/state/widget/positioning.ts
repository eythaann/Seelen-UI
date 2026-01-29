import { Alignment, type Frame, type PhysicalMonitor } from "@seelen-ui/types";
import { invoke, subscribe } from "../../handlers/mod.ts";
import { SeelenCommand } from "@seelen-ui/lib";
import { SeelenEvent } from "../../handlers/events.ts";

interface args {
  frame: Frame;
  alignX?: Alignment | null;
  alignY?: Alignment | null;
}

const monitors = {
  value: [] as PhysicalMonitor[],
};

export async function initMonitorsState(): Promise<void> {
  monitors.value = await invoke(SeelenCommand.SystemGetMonitors);
  subscribe(SeelenEvent.SystemMonitorsChanged, ({ payload }) => {
    monitors.value = payload;
  });
}

function monitorFromPoint(x: number, y: number): PhysicalMonitor | undefined {
  return monitors.value.find(
    (m) => m.rect.left <= x && x < m.rect.right && m.rect.top <= y && y < m.rect.bottom,
  );
}

function primaryMonitor(): PhysicalMonitor | undefined {
  return monitors.value.find((m) => m.isPrimary);
}

export function adjustPositionByPlacement({
  frame: { x, y, width, height },
  alignX,
  alignY,
}: args): Frame {
  if (alignX === Alignment.Center) {
    x -= width / 2;
  } else if (alignX === Alignment.Start) {
    x -= width;
  }

  if (alignY === Alignment.Center) {
    y -= height / 2;
  } else if (alignY === Alignment.Start) {
    y -= height;
  }

  const newFrame = fitIntoMonitor({ x, y, width, height });
  return {
    x: Math.round(newFrame.x),
    y: Math.round(newFrame.y),
    width: Math.round(newFrame.width),
    height: Math.round(newFrame.height),
  };
}

export function fitIntoMonitor({ x, y, width, height }: Frame): Frame {
  const monitor = monitorFromPoint(Math.round(x), Math.round(y)) || primaryMonitor();
  if (monitor) {
    width = Math.min(width, monitor.rect.right - monitor.rect.left);
    height = Math.min(height, monitor.rect.bottom - monitor.rect.top);

    const x2 = x + width;
    const y2 = y + height;

    // check left edge
    if (x < monitor.rect.left) {
      x = monitor.rect.left;
    }

    // check top edge
    if (y < monitor.rect.top) {
      y = monitor.rect.top;
    }

    // check right edge
    if (x2 > monitor.rect.right) {
      x = monitor.rect.right - width;
    }

    // check bottom edge
    if (y2 > monitor.rect.bottom) {
      y = monitor.rect.bottom - height;
    }
  }

  return {
    x,
    y,
    width,
    height,
  };
}
