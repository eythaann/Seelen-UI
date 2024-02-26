import { createSlice } from '@reduxjs/toolkit';

import { Rect } from '../../shared/app/Rect';
import { matcher, reducersFor, selectorsFor } from '../../shared/app/utils';
import { AnimationsSlice } from '../animations/app';
import { BorderSlice } from '../border/app';
import { ContainerTopBarSlice } from '../containerTopBar/app';
import { PopupSlice } from '../popups/app';

import {
  CrossMonitorMoveBehaviour,
  GeneralSettingsState,
  UnmanagedWindowOperationBehaviour,
  WindowContainerBehaviour,
  WindowHidingBehaviour,
} from './domain';

let initialState: GeneralSettingsState = {
  altFocusHack: false,
  autoStackinByCategory: true,
  animations: AnimationsSlice.getInitialState(),
  border: BorderSlice.getInitialState(),
  popups: PopupSlice.getInitialState(),
  containerPadding: 10,
  workspacePadding: 10,
  resizeDelta: 50,
  mouseFollowFocus: false,
  focusFollowsMouse: null,
  windowContainerBehaviour: WindowContainerBehaviour.CREATE,
  windowHidingBehaviour: WindowHidingBehaviour.MINIMIZE,
  invisibleBorders: {
    left: 7,
    right: 14,
    top: 0,
    bottom: 7,
  },
  globalWorkAreaOffset: new Rect().plain(),
  unmanagedWindowOperationBehaviour: UnmanagedWindowOperationBehaviour.OP,
  crossMonitorMoveBehaviour: CrossMonitorMoveBehaviour.SWAP,
  containerTopBar: ContainerTopBarSlice.getInitialState(),
  monitorIndexPreferences: null,
  displayindexpreferences: null,
};

export const GeneralSettingsSlice = createSlice({
  name: 'generalSettings',
  initialState,
  selectors: selectorsFor(initialState),
  reducers: {
    ...reducersFor(initialState),
  },
  extraReducers: (builder) => {
    builder
      .addMatcher(matcher(BorderSlice), (state, action) => {
        state.border = BorderSlice.reducer(state.border, action);
      })
      .addMatcher(matcher(PopupSlice), (state, action) => {
        state.popups = PopupSlice.reducer(state.popups, action);
      })
      .addMatcher(matcher(AnimationsSlice), (state, action) => {
        state.animations = AnimationsSlice.reducer(state.animations, action);
      })
      .addMatcher(matcher(ContainerTopBarSlice), (state, action) => {
        state.containerTopBar = ContainerTopBarSlice.reducer(state.containerTopBar, action);
      });
  },
});

export const GeneralSettingsActions = GeneralSettingsSlice.actions;