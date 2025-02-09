import { SeelenCommand } from '@seelen-ui/lib';
import { ToolbarItem } from '@seelen-ui/lib/types';
import { invoke } from '@tauri-apps/api/core';
import { TFunction } from 'i18next';
import { evaluate } from 'mathjs';
import { Dispatch } from 'redux';

import { SaveToolbarItems } from '../main/application';
import { RootActions } from '../shared/store/app';
import { Icon } from 'src/apps/shared/components/Icon';

export class Scope {
  scope: Map<string, any>;

  constructor() {
    this.scope = new Map();
  }

  get(key: string) {
    return this.scope.get(key);
  }

  set(key: string, value: any) {
    return this.scope.set(key, value);
  }

  has(key: string) {
    return this.scope.has(key);
  }

  keys(): string[] | IterableIterator<string> {
    return this.scope.keys();
  }

  loadInvokeActions() {
    for (const [key, value] of Object.entries(ActionsScope)) {
      this.set(key, value);
    }
  }
}

const ActionsScope = {
  open(path: string) {
    invoke(SeelenCommand.OpenFile, { path }).catch(console.error);
  },
  run(program: string, args: string[], workingDir: string) {
    invoke(SeelenCommand.Run, { program, args, workingDir }).catch(console.error);
  },
  copyClipboard(text: string) {
    navigator.clipboard.writeText(text);
  },
};

export function safeEval(expression: string, scope: Scope) {
  try {
    evaluate(expression, scope);
  } catch (error) {
    console.error(error);
  }
}

export function CommonItemContextMenu(t: TFunction, d: Dispatch, item: Omit<ToolbarItem, 'type'>) {
  return [
    {
      key: 'remove',
      label: t('context_menu.remove'),
      icon: <Icon iconName="CgExtensionRemove" />,
      className: 'ft-bar-item-context-menu-item',
      onClick() {
        d(RootActions.removeItem(item.id));
        SaveToolbarItems()?.catch(console.error);
      },
    },
  ];
}