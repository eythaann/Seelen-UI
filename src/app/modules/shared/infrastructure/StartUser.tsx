import { SaveStore } from './store';
import { Modal } from 'antd';

import cs from './StartUser.module.css';

export const StartUser = () => {
  const showMigrationInfo = () => {
    Modal.info({
      title: 'Migration from Komorebi CLI',
      className: cs.welcome,
      content: (
        <div>
          <p>
            If you are migrating from komorebi cli, you can load your old configs in the
            information tab, also try remove or unistalling the old version to avoid any
            type of issues or conflicts, good luck!.
          </p>
        </div>
      ),
      okText: 'Ok',
      cancelButtonProps: { style: { display: 'none' } },
      centered: true,
    });
  };

  const welcome = Modal.confirm({
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
      welcome.destroy();
      showMigrationInfo();
    },
    icon: <div className={cs.icon}>🎉</div>,
    cancelButtonProps: { style: { display: 'none' } },
    centered: true,
  });
};
