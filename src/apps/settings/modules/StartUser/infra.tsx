import { Modal } from 'antd';

import { SaveStore, store } from '../shared/store/infra';
import { startup } from '../shared/tauri/infra';

import { RootActions } from '../shared/store/app/reducer';

import cs from './index.module.css';

export const StartUser = () => {
  startup.enable();
  store.dispatch(RootActions.setAutostart(true));

  const modal = Modal.confirm({
    title: 'Welcome!',
    className: cs.welcome,
    content: (
      <div>
        <p>
          Welcome to Seelen UI, the ultimate Desktop Enviroment with an incorporated tiling windows manager
          to enhance your Windows 11 experience! Explore a new era of efficiency and multitasking with our \
          intuitive interface and advanced features.
        </p>
        <b>Optimize your productivity with style!</b>
      </div>
    ),
    okText: 'Continue',
    onOk: () => {
      SaveStore();
      modal.destroy();
    },
    icon: <div className={cs.icon}>🎉</div>,
    cancelButtonProps: { style: { display: 'none' } },
    centered: true,
  });
};
