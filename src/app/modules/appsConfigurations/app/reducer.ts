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
    deleteMany: (state, action: PayloadAction<number[]>) => {
      const newState: any[] = [...state];
      action.payload.forEach((key) => {
        newState[key] = undefined;
      });
      return newState.filter(Boolean);
    },
    push: (state, action: PayloadAction<AppConfiguration[]>) => {
      state.push(...action.payload);
    },
    replace: (state, action: PayloadAction<AppPayload & { app: AppConfiguration }>) => {
      const { idx, app } = action.payload;
      state[idx] = app;
    },
    swap: (state, action: PayloadAction<[number, number]>) => {
      const [idx1, idx2] = action.payload;
      const App1 = state[idx1];
      const App2 = state[idx2];

      if (App1 && App2) {
        state[idx1] = App2;
        state[idx2] = App1;
      }
    },
  },
});

export const AppsConfigActions = AppsConfigSlice.actions;
