import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { getRootContainer } from "@shared";

interface Options {
  onResize?: () => void;
}

/** This function will update the size of the webview/window based on the size of the root element */
export function autoSizeWebviewBasedOnContent(options: Options = {}) {
  const { onResize } = options;

  const webview = getCurrentWebviewWindow();
  const root = getRootContainer();

  const updateSize = async () => {
    const contentWidth = Math.floor(root.scrollWidth);
    const contentHeight = Math.floor(root.scrollHeight);

    if (contentWidth > 0 && contentHeight > 0) {
      const { width: currentWidth, height: currentHeight } = await webview.outerSize();
      const currentWidthFloored = Math.floor(currentWidth);
      const currentHeightFloored = Math.floor(currentHeight);

      // Only update if the difference is more than 1px (avoid infinite loops from decimal differences)
      if (
        Math.abs(contentWidth - currentWidthFloored) > 1 ||
        Math.abs(contentHeight - currentHeightFloored) > 1
      ) {
        const size = new LogicalSize(contentWidth, contentHeight);
        webview.setSize(size).catch((err) => {
          console.error("Failed to update webview size:", err);
        });
        onResize?.();
      }
    }
  };

  // Initial size update
  updateSize();

  // Update size when content changes
  const observer = new MutationObserver(() => {
    updateSize();
  });

  observer.observe(root, {
    childList: true,
    subtree: true,
    attributes: true,
    characterData: true,
  });

  // Also update on window resize (for responsive content)
  window.addEventListener("resize", updateSize);

  // Cleanup function
  return () => {
    observer.disconnect();
    window.removeEventListener("resize", updateSize);
  };
}
