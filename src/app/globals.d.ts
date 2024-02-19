
declare module '*.module.css' {
  const classnames: Record<string, string>;
  export default classnames;
}

interface BackgroundApi {
  enableAutostart: () => Promise<void>;
  disableAutostart: () => Promise<void>;
  autostartTaskExist: () => Promise<boolean>;
}

interface Window {
  backgroundApi: BackgroundApi;
}