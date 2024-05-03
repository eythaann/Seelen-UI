import { ToolbarModuleType } from './apps/utils/schemas/Placeholders';

declare global {
  interface Window {
    TOOLBAR_MODULES: Record<ToolbarModuleType, boolean>;
  }

  declare module '*.module.css' {
    const classnames: Record<string, string>;
    export default classnames;
  }
}
