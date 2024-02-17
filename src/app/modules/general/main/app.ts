import { createSlice } from '@reduxjs/toolkit';

import { Rect } from '../../shared/app/Rect';
import { matcher, selectorsFor } from '../../shared/app/utils';
import { AnimationsSlice } from '../animations/app';
import { BorderSlice } from '../border/app';
import { ContainerTopBarSlice } from '../containerTopBar/app';

import { CrossMonitorMoveBehaviour, FocusFollowsMouse, GeneralSettingsState, UnmanagedWindowOperationBehaviour, WindowContainerBehaviour, WindowHidingBehaviour } from './domain';

let initialState: GeneralSettingsState = {
  altFocusHack: false,
  autoStackinByCategory: true,
  animations: AnimationsSlice.getInitialState(),
  border: BorderSlice.getInitialState(),
  containerPadding: 10,
  workspacePadding: 10,
  resizeDelta: 50,
  mouseFollowFocus: false,
  focusFollowsMouse: FocusFollowsMouse.NONE,
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
    toggleAltFocuseHack: (state) => {
      state.altFocusHack = !state.altFocusHack;
    },
    toggleAutoStackinByCategory: (state) => {
      state.autoStackinByCategory = !state.autoStackinByCategory;
    },
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