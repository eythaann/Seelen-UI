import { SaveStore } from './store';
import { Modal } from 'antd';

import cs from './StartUser.module.css';

const showNewUserTutorial = () => {
  const modal = Modal.confirm({
    title: 'Dependencies',
    className: cs.welcome,
    content: (
      <div>
        <p>
          Komorebi UI needs AutoHotKey to work.
        </p>
      </div>
    ),
    okText: 'Install',
    onOk: () => {
      // TODO(eythan) window.backgroundApi.runAhkSetup();
      modal.destroy();
    },
    cancelText: 'Omit',
    centered: true,
  });
};

const showMigrationInfo = () => {
  const modal = Modal.info({
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
    onOk: () => {
      modal.destroy();
      showNewUserTutorial();
    },
    cancelButtonProps: { style: { display: 'none' } },
    centered: true,
  });
};

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
      showMigrationInfo();
    },
    icon: <div className={cs.icon}>🎉</div>,
    cancelButtonProps: { style: { display: 'none' } },
    centered: true,
  });
};
