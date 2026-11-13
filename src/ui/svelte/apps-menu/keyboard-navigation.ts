export type NavigationDirection = "up" | "down" | "left" | "right";

const arrowDirections = new Map<string, NavigationDirection>([
  ["ArrowUp", "up"],
  ["ArrowDown", "down"],
  ["ArrowLeft", "left"],
  ["ArrowRight", "right"],
]);

export interface SelectionScope {
  container: () => ParentNode;
  getSelected: () => string | null;
  setSelected: (id: string | null) => void;
}

function getItems(scope: SelectionScope): HTMLElement[] {
  return Array.from(scope.container().querySelectorAll<HTMLElement>(".app, .folder"));
}

export function navigateInDirection(
  direction: NavigationDirection,
  scope: SelectionScope,
): void {
  const allItems = getItems(scope);
  if (allItems.length === 0) return;

  const selected = scope.getSelected();

  const focused = allItems.find((item) => item === item.ownerDocument.activeElement);
  const currentElement = focused ?? allItems.find((item) => item.dataset.itemId === selected);
  if (!currentElement) {
    scope.setSelected(allItems[0]?.dataset.itemId || null);
    return;
  }

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

  const idxToTake = ["right", "down"].includes(direction) ? 0 : -1;
  const toTake = candidates.at(idxToTake);
  if (toTake) {
    scope.setSelected(toTake.dataset.itemId || null);
    if (focused) {
      toTake.focus({ preventScroll: true });
    }
  }
}

export function handleGridKeyDown(event: KeyboardEvent, scope: SelectionScope): void {
  const target = event.target;
  if (
    event.defaultPrevented || event.isComposing || !(target instanceof HTMLElement) ||
    !target.matches('input[type="search"], dialog, .app, .folder')
  ) {
    return;
  }

  if (event.key === "Enter") {
    const selected = scope.getSelected();
    event.preventDefault();
    if (selected !== null) {
      getItems(scope).find((item) => item.dataset.itemId === selected)?.click();
    }
    return;
  }

  const direction = arrowDirections.get(event.key);
  if (direction) {
    event.preventDefault();
    navigateInDirection(direction, scope);
  }
}

export interface InputKeyDownOptions {
  // Return true to handle Enter here instead of activating the selected item.
  onEnter?: (event: KeyboardEvent, input: HTMLInputElement) => boolean | void;
}

// Keydown handler for the search input. It only deals with input-specific
// concerns; ArrowUp/ArrowDown/ArrowLeft and non-web Enter are left to bubble
// up to whichever view's own window-level listener is currently mounted, so
// this input never needs to know which view (or its selection) is active.
export function createInputKeyDownHandler(options: InputKeyDownOptions = {}) {
  return function handleInputKeyDown(event: KeyboardEvent) {
    const input = event.currentTarget as HTMLInputElement;

    switch (event.key) {
      case "Enter": {
        if (options.onEnter?.(event, input)) {
          event.preventDefault();
          event.stopPropagation();
        }
        break;
      }
      case "ArrowRight":
        // Only hand off to grid navigation once the cursor is already at the
        // end; otherwise let the default caret move happen.
        if (input.selectionStart !== input.value.length) {
          event.stopPropagation();
        }
        break;
    }
  };
}
