export function removeDefaultWebviewActions(): void {
  globalThis.addEventListener('keydown', function (event): void {
    // Prevent refresh
    if (event.key === 'F5' || (event.ctrlKey && event.key === 'r')) {
      event.preventDefault();
    }

    // Prevent closing the window (Alt+F4 / Cmd+Q on macOS)
    if ((event.altKey && event.key === 'F4') || (event.metaKey && event.key === 'q')) {
      event.preventDefault();
    }

    // Prevent common Ctrl/Cmd shortcuts
    if (event.ctrlKey || event.metaKey) {
      switch (event.key) {
        case 'n': // New window
        case 't': // New tab
        case 'w': // Close tab
        case 'f': // Find
        case 'g': // Find next
        case 'p': // print
        case 's': // Save
        case 'o': // Open file
        case 'j': // downloads
        case 'u': // View source
        case 'tab': // Switch tabs
          event.preventDefault();
          break;
      }
    }
  });

  // prevent browser context menu
  globalThis.addEventListener('contextmenu', (e) => e.preventDefault(), {
    capture: true,
  });

  // Prevent drag-and-drop (files, links, images)
  globalThis.addEventListener('drop', (e) => e.preventDefault());
  globalThis.addEventListener('dragover', (e) => e.preventDefault());
  globalThis.addEventListener('dragstart', (e) => e.preventDefault());
}
