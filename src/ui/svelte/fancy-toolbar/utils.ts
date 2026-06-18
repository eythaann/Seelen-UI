import type { ToolbarItem2 } from "@seelen-ui/lib/types";

export function matchIds(tb: ToolbarItem2, id: string): boolean {
  if (typeof tb === "string") {
    return tb === id;
  }
  return tb.id === id;
}

export function styleToString(style: Record<string, any>): string {
  return Object.entries(style)
    .filter(([, v]) => v !== null && v !== undefined)
    .map(([k, v]) => {
      const cssKey = k.replace(/([A-Z])/g, "-$1").toLowerCase();
      return `${cssKey}: ${v}`;
    })
    .join("; ");
}
