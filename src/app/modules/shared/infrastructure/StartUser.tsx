import { SaveStore } from './store';
import { Modal } from 'antd';

import cs from './StartUser.module.css';

export const StartUser = () => {
  const modal = Modal.confirm({
    title: 'Welcome!',
    className: cs.welcome,
    content: (
      <div>
        <p>
          Welcome to Komorebi-UI, the ultimate tiling windows manager to enhance your Windows 11 experience! Explore a
          new era of efficiency and multitasking with our intuitive interface and advanced features.
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
