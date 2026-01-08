import { globalState } from "./state.svelte";

export function navigateInDirection(direction: "up" | "down" | "left" | "right"): void {
  const allItems = Array.from(document.querySelectorAll(".app")) as HTMLElement[];
  if (allItems.length === 0) return;

  let currentElement: HTMLElement | null = null;

  if (globalState.preselectedItem) {
    currentElement = allItems.find((item) => item.dataset.itemId === globalState.preselectedItem) || null;
  } else {
    currentElement = allItems[0] || null;
  }

  if (!currentElement) return;

  const currentRect = currentElement.getBoundingClientRect();
  const candidates = allItems
    // filter items that are not in the same row/column
    .filter((item) => {
      if (item === currentElement) return false;
      const rect = item.getBoundingClientRect();
      switch (direction) {
        case "right":
          return rect.top === currentRect.top && rect.left > currentRect.left;
        case "left":
          return rect.top === currentRect.top && rect.left < currentRect.left;
        case "down":
          return rect.left === currentRect.left && rect.top > currentRect.top;
        case "up":
          return rect.left === currentRect.left && rect.top < currentRect.top;
      }
    });

  let idxToTake = ["right", "down"].includes(direction) ? 0 : -1;
  let toTake = candidates.at(idxToTake);
  if (toTake) {
    globalState.preselectedItem = toTake.dataset.itemId || null;
  }
}
