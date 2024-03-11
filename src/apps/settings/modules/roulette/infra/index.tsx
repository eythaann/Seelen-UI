import cs from './index.module.css';

export function Roulette() {
  return <div className={cs.roulette}>
    <div className={cs.drag} data-tauri-drag-region>➕</div>
  </div>;
}