import { getCurrentWindow } from '@tauri-apps/api/window';

export * from './hooks';
export * from './layered_hitbox';

export function getRootElement() {
  const element = document.getElementById('root');
  if (!element) {
    throw new Error('Root element not found');
  }
  return element;
}

export class Rect {
  left = 0;
  top = 0;
  right = 0;
  bottom = 0;
}

export function disableWebviewShortcutsAndContextMenu() {
  window.addEventListener('keydown', function (event) {
    // prevent refresh
    if (event.key === 'F5') {
      event.preventDefault();
    }

    // prevent close
    if (event.altKey && event.key === 'F4') {
      event.preventDefault();
    }

    // others
    if (event.ctrlKey) {
      switch (event.key) {
        case 'r': // reload
        case 'f': // search
        case 'g': // find
        case 'p': // print
        case 'j': // downloads
        case 'u': // source
          event.preventDefault();
          break;
      }
    }
  });
  window.addEventListener('contextmenu', (e) => e.preventDefault(), { capture: true });
  window.addEventListener('drop', (e) => e.preventDefault());
  window.addEventListener('dragover', (e) => e.preventDefault());
}

// label schema: user/resource__query__monitor:display5
export function getCurrentWidget() {
  const { label } = getCurrentWindow();
  const parsedLabel = label.replace('__query__', '?').replace(':', '=');
  const query = new URLSearchParams(parsedLabel);
  return {
    id: `@${parsedLabel.split('?')[0]}`,
    label,
    attachedMonitor: query.get('monitor'),
  };
}