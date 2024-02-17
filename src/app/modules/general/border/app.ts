import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { defaultOnNull, selectorsFor } from '../../shared/app/utils';

import { HexColor } from '../../shared/domain/interfaces';
import { BorderState } from './domain';

const initialState: BorderState = {
  enable: false,
  offset: 0,
  width: 20,
  color: '#ff0000',
};

export const BorderSlice = createSlice({
  name: 'generalSettings/border',
  initialState,
  reducers: {
    toggleEnable: (state) => {
      state.enable = !state.enable;
    },
    updateOffset: (state, action: PayloadAction<number | null>) => {
      state.offset = defaultOnNull(action.payload, initialState.offset);
    },
    updateWidth: (state, action: PayloadAction<number | null>) => {
      state.width = defaultOnNull(action.payload, initialState.width);
    },
    updateColor: (state, action: PayloadAction<HexColor | null>) => {
      state.color = defaultOnNull(action.payload, initialState.color);
    },
  },
  selectors: selectorsFor(initialState),
});

export const BorderActions = BorderSlice.actions;