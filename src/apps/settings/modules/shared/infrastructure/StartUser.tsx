import { SaveStore, store } from './store';
import { startup } from './tauri';
import { invoke } from '@tauri-apps/api/core';
import { Modal } from 'antd';

import { RootActions } from '../app/reducer';

import cs from './StartUser.module.css';

const showNewUserTutorial = () => {
  const modal = Modal.confirm({
    title: 'Dependencies',
    className: cs.welcome,
    content: (
      <div>
        <p>
          Komorebi UI by default needs AutoHotKey to work.
          You can omit this step if you will configure your own shortcuts system.
          Please restart your pc after install AutoHotKey.
        </p>
      </div>
    ),
    okText: 'Install',
    onOk: async () => {
      invoke('run_ahk_installer');
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
          If you are migrating from komorebi cli, you can load your old configs in the information
          tab, also try remove or unistalling the old version to avoid any type of issues or
          conflicts, good luck!.
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
  startup.enable();
  store.dispatch(RootActions.setAutostart(true));

  const modal = Modal.confirm({
    title: 'Welcome!',
    className: cs.welcome,
    content: (
      <div>
        <p>
          Welcome to Komorebi-UI, the ultimate tiling windows manager to enhance your Windows 11
          experience! Explore a new era of efficiency and multitasking with our intuitive interface
          and advanced features.
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
