import { monitorFromPoint } from "@tauri-apps/api/window";
import { Alignment } from "@seelen-ui/lib/types";

interface args {
  x: number;
  y: number;
  width: number;
  height: number;
  alignX?: Alignment | null;
  alignY?: Alignment | null;
}

export async function adjustPostionByPlacement({
  x,
  y,
  width,
  height,
  alignX,
  alignY,
}: args): Promise<{ x: number; y: number }> {
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

  const fixed = await fitIntoMonitor({ x, y, width, height });
  return {
    x: Math.round(fixed.x),
    y: Math.round(fixed.y),
  };
}

async function fitIntoMonitor({
  x,
  y,
  width,
  height,
}: {
  x: number;
  y: number;
  width: number;
  height: number;
}): Promise<{ x: number; y: number }> {
  const monitor = await monitorFromPoint(x, y);
  if (monitor) {
    const x2 = x + width;
    const y2 = y + height;

    const mx = monitor.position.x;
    const my = monitor.position.y;
    const mx2 = mx + monitor.size.width;
    const my2 = my + monitor.size.height;

    // check left edge
    if (x < mx) {
      x = mx;
    }

    // check top edge
    if (y < my) {
      y = my;
    }

    // check right edge
    if (x2 > mx2) {
      x = mx2 - width;
    }

    // check bottom edge
    if (y2 > my2) {
      y = my2 - height;
    }

    // ensure final position is still within monitor bounds (in case window is larger than monitor)
    if (x < mx) {
      x = mx;
    }

    if (y < my) {
      y = my;
    }
  }

  return {
    x,
    y,
  };
}
