declare module '*.module.css' {
  const classnames: Record<string, string>;
  export default classnames;
}

declare module '*.module.scss' {
  const classnames: Record<string, string>;
  export default classnames;
}

declare module '*.yml' {
  export default string;
}

interface ObjectConstructor {
  keys<T>(o: T): (T extends any ? keyof T : PropertyKey)[];
}

interface Window {
  __TAURI_INTERNALS__: {
    metadata?: {
      currentWebview?: {
        label?: string;
      };
    };
    invoke: any;
  };
}
