import { configureStore } from "@reduxjs/toolkit";

import { RootSlice } from "./app";

export const store = configureStore({
  reducer: RootSlice.reducer,
  middleware(getDefaultMiddleware) {
    return getDefaultMiddleware({
      serializableCheck: false,
    });
  },
});
