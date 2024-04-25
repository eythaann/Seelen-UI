import cs from './index.module.css';

export function ErrorFallback() {
  return <div className={cs.error}>Something went wrong - please restart the app</div>;
}
