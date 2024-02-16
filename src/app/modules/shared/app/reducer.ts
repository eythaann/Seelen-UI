import { createSlice } from '@reduxjs/toolkit';

import { Route } from '../domain/routes';

let initialState = {
  route: Route.GENERAL,
};

const slice = createSlice({
  name: 'main',
  initialState,
  reducers: {
    setRoute: (state, action) => {
      state.route = action.payload;
    },
  },
});

export const mainReducer = slice.reducer;
export const mainActions = slice.actions;
