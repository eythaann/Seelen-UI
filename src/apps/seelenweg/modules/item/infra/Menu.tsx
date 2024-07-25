import { store } from '../../shared/store/infra';

import { RootActions } from '../../shared/store/app';

export const MediaSessionMenu = [
  {
    key: 'remove',
    label: 'Remove Media Module',
    onClick() {
      store.dispatch(RootActions.removeMediaModule());
    },
  },
];

export const StartModuleMenu = [
  {
    key: 'remove',
    label: 'Remove Start Module',
    onClick() {
      store.dispatch(RootActions.removeStartModule());
    },
  },
];