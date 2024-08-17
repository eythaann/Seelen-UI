import { parseAsCamel } from '../../../shared/schemas';
import { AhkVar, AhkVariables, AhkVariablesSchema } from '../../../shared/schemas/Settings';
import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { selectorsFor } from '../shared/utils/app';

function getAHK(code: string): string | undefined {
  if (code.startsWith('Key')) {
    return code.replace('Key', '');
  }

  if (code.startsWith('Digit')) {
    return code.replace('Digit', '');
  }

  if (code.startsWith('Arrow')) {
    return code.replace('Arrow', '');
  }

  if (/F[0-9]+$/.test(code)) {
    return code;
  }

  if (/Numpad[0-9]$/.test(code)) {
    return code;
  }

  return;
}

export function KeyCodeToAHK(e: React.KeyboardEvent<HTMLInputElement>) {
  e.preventDefault();
  let fancy = '';
  let ahk = '';

  if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) {
    return;
  }

  let key = getAHK(e.code);
  if (!key) {
    return;
  }

  if (e.metaKey) {
    fancy += 'Win + ';
    ahk += '#';
  }

  if (e.ctrlKey) {
    fancy += 'Ctrl + ';
    ahk += '^';
  }

  if (e.altKey) {
    fancy += 'Alt + ';
    ahk += '!';
  }

  if (e.shiftKey) {
    fancy += 'Shift + ';
    ahk += '+';
  }

  fancy += key;
  ahk += key.toLocaleLowerCase();

  return {
    fancy,
    ahk,
  };
}

const initialState: AhkVariables = parseAsCamel(AhkVariablesSchema, {});

export const AhkVariablesSlice = createSlice({
  name: 'AhkVariables',
  initialState,
  selectors: selectorsFor(initialState),
  reducers: {
    setVariable(state, action: PayloadAction<{ name: string; value: AhkVar }>) {
      state[action.payload.name] = action.payload.value;
    },
  },
});

export const AhkVariablesActions = AhkVariablesSlice.actions;