import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { AppConfiguration } from '../domain';

const initialState: AppConfiguration[] = [];

interface AppPayload {
  idx: number;
}

export const AppsConfigSlice = createSlice({
  name: 'monitors',
  initialState,
  reducers: {
    delete: (state, action: PayloadAction<number>) => {
      state.splice(action.payload, 1);
    },
    push: (state, action: PayloadAction<AppConfiguration>) => {
      state.push(action.payload);
    },
    replace: (state, action: PayloadAction<AppPayload & { app: AppConfiguration }>) => {
      const { idx, app } = action.payload;
      state[idx] = app;
    },
  },
});

export const AppsConfigActions = AppsConfigSlice.actions;