export function removeDefaultWebviewActions(): void {
  globalThis.addEventListener("keydown", function (event): void {
    // Prevent refresh
    if (event.key === "F5" || (event.ctrlKey && event.key === "r")) {
      event.preventDefault();
    }

    // Prevent closing the window (Alt+F4 / Cmd+Q on macOS)
    if ((event.altKey && event.key === "F4") || (event.metaKey && event.key === "q")) {
      event.preventDefault();
    }

    // Prevent common Ctrl/Cmd shortcuts
    if (event.ctrlKey || event.metaKey) {
      switch (event.key) {
        case "n": // New window
        case "t": // New tab
        case "w": // Close tab
        case "f": // Find
        case "g": // Find next
        case "p": // print
        case "s": // Save
        case "o": // Open file
        case "j": // downloads
        case "u": // View source
        case "tab": // Switch tabs
          event.preventDefault();
          break;
      }
    }
  });

  // prevent browser context menu
  globalThis.addEventListener("contextmenu", (e) => e.preventDefault(), {
    capture: true,
  });

  // Prevent drag-and-drop (files, links, images)
  globalThis.addEventListener("drop", (e) => e.preventDefault());
  globalThis.addEventListener("dragover", (e) => e.preventDefault());
  globalThis.addEventListener("dragstart", (e) => e.preventDefault());
}

export function applyUserExperienceImprovements(): void {
  document.addEventListener("keydown", (e: KeyboardEvent) => {
    if (e.defaultPrevented) return;
    if (e.key !== "Enter" && e.key !== " ") return;

    const target = e.target as HTMLElement;
    if (target.getAttribute("role") !== "button") return;

    target.dataset["ux-active"] = "true";
  });

  document.addEventListener("keyup", (e: KeyboardEvent) => {
    if (e.defaultPrevented) return;
    if (e.key !== "Enter" && e.key !== " ") return;

    const target = e.target as HTMLElement;
    if (target.getAttribute("role") !== "button") return;

    if (target.dataset["ux-active"] !== "true") {
      target.dataset["ux-active"] = "false";

      if ("click" in target) {
        target.click();
      }
    }
  });
}

/** The purpose of this is avoid collition of keys, taking in care that all widgets share same origin */
export function hookLocalStorage(widgetId: string) {
  const nativeLocalStorage = window.localStorage;

  class MyLocalStorage implements Storage {
    get length() {
      return nativeLocalStorage.length;
    }

    setItem(key: string, value: string) {
      nativeLocalStorage.setItem(`${widgetId}:${key}`, value);
    }

    getItem(key: string) {
      return nativeLocalStorage.getItem(`${widgetId}:${key}`);
    }

    removeItem(key: string) {
      nativeLocalStorage.removeItem(`${widgetId}:${key}`);
    }

    key(index: number) {
      return nativeLocalStorage.key(index);
    }

    clear() {
      nativeLocalStorage.clear();
    }
  }

  Object.defineProperty(window, "localStorage", {
    value: new MyLocalStorage(),
    writable: true,
  });
}
