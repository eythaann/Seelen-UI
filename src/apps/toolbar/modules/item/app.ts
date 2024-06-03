import { invoke } from '@tauri-apps/api/core';
import { evaluate } from 'mathjs';

export enum Actions {
  Open = 'open',
  CopyToClipboard = 'copy-to-clipboard',
  SwitchWorkspace = 'switch-workspace',
}

export function performClick(onClick: string | null, scope: any) {
  if (!onClick) {
    return;
  }

  const [_action, _argument] = onClick.split('->');
  const action = _action?.trim();
  const argument = _argument?.trim();

  if (!action) {
    return;
  }

  switch (action) {
    case Actions.Open:
      if (argument) {
        invoke('open_file', { path: evaluate(argument, scope) });
      }
      break;
    case Actions.CopyToClipboard:
      if (argument) {
        navigator.clipboard.writeText(evaluate(argument, scope));
      }
    case Actions.SwitchWorkspace:
      if (argument) {
        invoke('switch_workspace', { idx: evaluate(argument, scope) });
      }
  }
}