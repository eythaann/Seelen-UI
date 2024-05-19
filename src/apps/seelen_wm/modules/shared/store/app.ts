import { defaultLayout, defaultTheme } from '../../../../../shared.interfaces';
import { toPhysicalPixels } from '../../../../utils';
import { parseAsCamel } from '../../../../utils/schemas';
import { WindowManagerSchema } from '../../../../utils/schemas/WindowManager';
import { StateBuilder } from '../../../../utils/StateBuilder';
import { createSlice, PayloadAction } from '@reduxjs/toolkit';
import { invoke } from '@tauri-apps/api/core';
import { cloneDeep } from 'lodash';

import { NodeImpl, reIndexContainer } from '../../layout/app';

import { Reservation, Sizing } from '../../layout/domain';
import { AddWindowPayload, DesktopId, FocusAction, RootState } from './domain';

const initialState: RootState = {
  version: 0,
  availableLayouts: [],
  workspaces: {},
  activeWorkspace: '' as DesktopId,
  desktopByHandle: {},
  handlesByDesktop: {},
  activeWindow: 0,
  lastManagedActivated: null,
  reservation: null,
  settings: parseAsCamel(WindowManagerSchema, {}),
  themeLayers: defaultTheme.layers,
};

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
    addWindow: (state, action: PayloadAction<AddWindowPayload>) => {
      const { desktop_id, hwnd, as_floating } = action.payload;

      state.desktopByHandle[hwnd] = desktop_id;
      state.handlesByDesktop[desktop_id] ??= [];

      const handlesInDesktop = state.handlesByDesktop[desktop_id]!;
      handlesInDesktop.push(hwnd);

      if (!state.workspaces[desktop_id]) {
        state.workspaces[desktop_id] = {
          name: `Workspace ${desktop_id}`,
          layout: cloneDeep(state.availableLayouts.find((l) => l.info.filename === state.settings.defaultLayout) || defaultLayout),
        };
      }

      const workspace = state.workspaces[desktop_id]!;
      const node = NodeImpl.from(workspace.layout.structure);

      let successfullyAdded = false;

      const setFloatingSize = () => {
        const top = toPhysicalPixels(window.screen.height / 2 - state.settings.floating.height / 2);
        const left = toPhysicalPixels(window.screen.width / 2 - state.settings.floating.width / 2);
        invoke('set_window_position', {
          hwnd,
          rect: {
            top,
            left,
            right: left + toPhysicalPixels(state.settings.floating.width),
            bottom: top + toPhysicalPixels(state.settings.floating.height),
          },
        });
      };

      if (state.reservation) {
        if (state.reservation === Reservation.Float) {
          invoke('bounce_handle', { hwnd });
          setFloatingSize();
          successfullyAdded = true;
        } else if (state.lastManagedActivated) {
          successfullyAdded = node.concreteReservation(
            hwnd,
            state.reservation,
            state.lastManagedActivated,
          );
        }
      } else if (as_floating) {
        setFloatingSize();
        successfullyAdded = true;
      } else {
        successfullyAdded = node.addHandle(hwnd);
        if (successfullyAdded) {
          reIndexContainer(node.inner, handlesInDesktop);
        }
      }

      state.reservation = null;

      if (successfullyAdded) {
        state.lastManagedActivated = hwnd;
        state.activeWindow = hwnd;
      } else {
        invoke('bounce_handle', { hwnd });
        if (!workspace.layout.noFallbackBehavior) {
          console.error('Layout can\'t handle the window, FallbackNode and noFallbackBehavior are not defined in layout');
        } else if (workspace.layout.noFallbackBehavior === 'Float') {
          setFloatingSize();
        }
      }
    },
    removeWindow: (state, action: PayloadAction<number>) => {
      const hwnd = action.payload;

      const desktopId = state.desktopByHandle[hwnd];
      if (!desktopId) {
        return;
      }

      delete state.desktopByHandle[hwnd];
      const handlesInDesktop = state.handlesByDesktop[desktopId] || [];
      const idx = handlesInDesktop.indexOf(hwnd);
      if (idx != -1) {
        handlesInDesktop.splice(idx, 1);
      }

      const workspace = state.workspaces[desktopId];
      if (workspace) {
        const node = NodeImpl.from(workspace.layout.structure);
        const wasRemoved = node.removeHandle(hwnd);
        if (wasRemoved) {
          reIndexContainer(node.inner, handlesInDesktop);
        }
      }
    },
    forceUpdate(state) {
      state.version += 1;
    },
    setActiveWorkspace(state, action: PayloadAction<DesktopId>) {
      state.activeWorkspace = action.payload;
      if (!state.workspaces[action.payload]) {
        state.workspaces[action.payload] = {
          name: `Workspace ${action.payload}`,
          layout: cloneDeep(state.availableLayouts.find((l) => l.info.filename === state.settings.defaultLayout) || defaultLayout),
        };
      }
    },
    updateSizing(state, action: PayloadAction<{ axis: 'x' | 'y'; sizing: Sizing }>) {
      const { axis, sizing } = action.payload;
      if (state.lastManagedActivated) {
        const node = NodeImpl.from(state.workspaces[state.activeWorkspace]!.layout.structure);
        node.updateGrowFactor(state.lastManagedActivated, axis, sizing);
      }
    },
    resetSizing(state) {
      const node = NodeImpl.from(state.workspaces[state.activeWorkspace]!.layout.structure);
      node.resetGrowFactor();
    },
    focus(state, action: PayloadAction<FocusAction>) {
      const { workspaces, activeWorkspace } = state;
      const workspace = workspaces[activeWorkspace];
      if (!workspace) {
        return console.error('No active workspace found.');
      }

      if (!state.lastManagedActivated) {
        return console.error('No last managed window found.');
      }

      if (action.payload === FocusAction.Latest) {
        invoke('request_focus', { hwnd: state.lastManagedActivated });
        return;
      }

      const node = NodeImpl.from(workspace.layout.structure);
      const next = node.getNodeAtSide(state.lastManagedActivated, action.payload);
      if (next) {
        const nextNode = NodeImpl.from(next);
        invoke('request_focus', { hwnd: nextNode.currentHandle || 0 });
      }
    },
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);
export const SelectCurrentWorkspace = (state: RootState) => state.workspaces[state.activeWorkspace];
