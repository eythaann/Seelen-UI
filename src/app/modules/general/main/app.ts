import { createSlice } from '@reduxjs/toolkit';

import { Rect } from '../../shared/app/Rect';
import { matcher, reducersFor, selectorsFor } from '../../shared/app/utils';
import { AnimationsSlice } from '../animations/app';
import { BorderSlice } from '../border/app';
import { ContainerTopBarSlice } from '../containerTopBar/app';

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
  containerPadding: 10,
  workspacePadding: 10,
  resizeDelta: 50,
  mouseFollowFocus: false,
  focusFollowsMouse: null,
  windowContainerBehaviour: WindowContainerBehaviour.CREATE,
  windowHidingBehaviour: WindowHidingBehaviour.MINIMIZE,
  invisibleBorders: new Rect().plain(),
  globalWorkAreaOffset: new Rect().plain(),
  unmanagedWindowOperationBehaviour: UnmanagedWindowOperationBehaviour.OP,
  crossMonitorMoveBehaviour: CrossMonitorMoveBehaviour.SWAP,
  containerTopBar: ContainerTopBarSlice.getInitialState(),
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
        BorderSlice.reducer(state.border, action);
      })
      .addMatcher(matcher(AnimationsSlice), (state, action) => {
        AnimationsSlice.reducer(state.animations, action);
      })
      .addMatcher(matcher(ContainerTopBarSlice), (state, action) => {
        ContainerTopBarSlice.reducer(state.containerTopBar, action);
      });
  },
});

export const GeneralSettingsActions = GeneralSettingsSlice.actions;