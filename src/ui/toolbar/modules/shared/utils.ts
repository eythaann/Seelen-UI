import { ToolbarItem2 } from "@seelen-ui/lib/types";

export function matchIds(tb: ToolbarItem2, id: string): boolean {
  if (typeof tb === "string") {
    return tb === id;
  }
  return tb.id === id;
}
