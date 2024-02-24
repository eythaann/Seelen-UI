export enum Channel {
  GET_AUTOSTART_STATUS = 'get-autostart-task',
  // actions
  ENABLE_AUTOSTART = 'enable-autostart',
  DISABLE_AUTOSTART = 'disable-autostart',
  QUIT = 'quit',
  RESTART = 'restart',
  // settings
  GET_USER_SETTINGS = 'get-user-settings',
  SAVE_USER_SETTINGS = 'save-user-settings',
  LOAD_APPS_TEMPLATE = 'load-apps-template',
  EXPORT_APPS_TEMPLATE = 'export-apps-template',
  // installers
  AHK_SETUP = 'ahk-setup',
}

type ReplayChannel = { [K in Channel]: `${K}-reply` };
export const REPLY_BY_CHANNEL: ReplayChannel = Object.values(Channel).reduce((acc: any, key) => {
  acc[key] = `${key}-reply`;
  return acc;
}, {});

