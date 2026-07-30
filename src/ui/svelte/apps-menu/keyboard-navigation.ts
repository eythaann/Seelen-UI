export type NavigationDirection = "up" | "down" | "left" | "right";

export interface SelectionScope {
  container: () => ParentNode;
  getSelected: () => string | null;
  setSelected: (id: string | null) => void;
}

export function navigateInDirection(
  direction: NavigationDirection,
  scope: SelectionScope,
): void {
  const allItems = Array.from(scope.container().querySelectorAll(".app, .folder")) as HTMLElement[];
  if (allItems.length === 0) return;

  const selected = scope.getSelected();

  if (!selected) {
    scope.setSelected(allItems[0]?.dataset.itemId || null);
    return;
  }

  const currentElement = allItems.find((item) => item.dataset.itemId === selected) || null;
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

  const idxToTake = ["right", "down"].includes(direction) ? 0 : -1;
  const toTake = candidates.at(idxToTake);
  if (toTake) {
    scope.setSelected(toTake.dataset.itemId || null);
  }
}

// Resolves the element that Enter should activate: the selected item, or the first one.
export function selectPreselectedOrFirst(scope: SelectionScope): HTMLElement | null {
  const selected = scope.getSelected();
  const root = scope.container();

  if (selected) {
    const element = root.querySelector<HTMLElement>(`[data-item-id="${selected}"]`);
    if (element) return element;
  }

  // return root.querySelector<HTMLElement>(".app, .folder");
  return null;
}

export interface InputKeyDownOptions {
  // return true to fully handle Enter here and skip the default "click
  // preselected or first item" behavior of whichever view is listening
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
