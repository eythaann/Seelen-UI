declare module '*.module.css' {
  const classnames: Record<string, string>;
  export default classnames;
}

interface Window {
  backgroundApi: import('../shared.interfaces').BackgroundApi;
}