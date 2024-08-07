import { ToolbarModuleType } from './apps/shared/schemas/Placeholders';

declare global {
  interface Window {
    TOOLBAR_MODULES: Record<ToolbarModuleType, boolean>;
  }

  declare module '*.module.css' {
    const classnames: Record<string, string>;
    export default classnames;
  }

  declare module '*.yml' {
    export default string;
  }

  interface ObjectConstructor {
    keys<T>(o: T): (keyof T)[];
  }
}
