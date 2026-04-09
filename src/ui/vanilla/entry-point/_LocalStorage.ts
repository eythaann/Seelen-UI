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
