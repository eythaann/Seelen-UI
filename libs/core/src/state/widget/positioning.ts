import { Alignment, type Frame, type PhysicalMonitor, type Point } from "@seelen-ui/types";
import { invoke, subscribe } from "../../handlers/mod.ts";
import { SeelenCommand } from "../../handlers/commands.ts";
import { SeelenEvent } from "../../handlers/events.ts";

interface args {
  frame: Frame;
  originX?: Alignment | null;
  originY?: Alignment | null;
  monitorPoint?: Point;
}

type MonitorBounds = Pick<PhysicalMonitor, "rect" | "isPrimary">;

const monitors = {
  value: [] as PhysicalMonitor[],
};

export async function initMonitorsState(): Promise<void> {
  monitors.value = await invoke(SeelenCommand.SystemGetMonitors);
  subscribe(SeelenEvent.SystemMonitorsChanged, ({ payload }) => {
    monitors.value = payload;
  });
}

export function adjustPositionByPlacement({
  frame: { x, y, width, height },
  originX,
  originY,
  monitorPoint = { x, y },
}: args, displays: readonly MonitorBounds[] = monitors.value): Frame {
  if (originX === Alignment.Center) {
    x -= width / 2;
  } else if (originX === Alignment.End) {
    x -= width;
  }

  if (originY === Alignment.Center) {
    y -= height / 2;
  } else if (originY === Alignment.End) {
    y -= height;
  }

  const newFrame = fitIntoMonitor({ x, y, width, height }, monitorPoint, displays);
  return {
    x: Math.round(newFrame.x),
    y: Math.round(newFrame.y),
    width: Math.round(newFrame.width),
    height: Math.round(newFrame.height),
  };
}

export function fitIntoMonitor(
  { x, y, width, height }: Frame,
  monitorPoint = { x, y },
  displays: readonly MonitorBounds[] = monitors.value,
): Frame {
  const monitor = displays.find(({ rect }) =>
    rect.left <= monitorPoint.x && monitorPoint.x < rect.right &&
    rect.top <= monitorPoint.y && monitorPoint.y < rect.bottom
  ) ?? displays.find((display) => display.isPrimary);
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
