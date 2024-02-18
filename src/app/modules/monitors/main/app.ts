import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { Layout } from '../layouts/domain';
import { Monitor } from './domain';

const initialState: Monitor[] = [{
  edditingWorkspace: 0,
  workAreaOffset: null,
  workspaces: [{
    name: 'Workspace 1',
    layout: Layout.BSP,
    layoutRules: null,
    containerPadding: null,
    workspacePadding: null,
    customLayout: null,
    customLayoutRules: null,
  }],
},
{
  edditingWorkspace: 0,
  workAreaOffset: null,
  workspaces: [{
    name: 'Workspace 1',
    layout: Layout.COLUMNS,
    layoutRules: null,
    containerPadding: null,
    workspacePadding: null,
    customLayout: null,
    customLayoutRules: null,
  }],
},
{
  edditingWorkspace: 0,
  workAreaOffset: null,
  workspaces: [{
    name: 'Workspace 1',
    layout: Layout.ROWS,
    layoutRules: null,
    containerPadding: null,
    workspacePadding: null,
    customLayout: null,
    customLayoutRules: null,
  }],
},
{
  edditingWorkspace: 0,
  workAreaOffset: null,
  workspaces: [{
    name: 'Workspace 1',
    layout: Layout.HORIZONTAL_STACK,
    layoutRules: null,
    containerPadding: null,
    workspacePadding: null,
    customLayout: null,
    customLayoutRules: null,
  }],
},
{
  edditingWorkspace: 0,
  workAreaOffset: null,
  workspaces: [{
    name: 'Workspace 1',
    layout: Layout.VERTICAL_STACK,
    layoutRules: null,
    containerPadding: null,
    workspacePadding: null,
    customLayout: null,
    customLayoutRules: null,
  }],
},
{
  edditingWorkspace: 0,
  workAreaOffset: null,
  workspaces: [{
    name: 'Workspace 1',
    layout: Layout.ULTRAWIDE_VERTICAL_STACK,
    layoutRules: null,
    containerPadding: null,
    workspacePadding: null,
    customLayout: null,
    customLayoutRules: null,
  }],
}];

interface ForMonitor {
  monitorIdx: number;
}

interface ForWorkspace extends ForMonitor {
  workspaceIdx: number;
}

export const MonitorsSlice = createSlice({
  name: 'monitors',
  initialState,
  reducers: {
    enableLayoutRules: (state, action: PayloadAction<ForWorkspace>) => {
      const { workspaceIdx, monitorIdx } = action.payload;
      let workspace = state[monitorIdx]?.workspaces[workspaceIdx];
      if (!workspace) {
        return;
      }
      workspace.layoutRules = {};
      for (let n = 1; n < 10; n++) {
        workspace.layoutRules[n] = workspace.layout;
      }
    },
    disableLayoutRules: (state, action: PayloadAction<ForWorkspace>) => {
      const { workspaceIdx, monitorIdx } = action.payload;
      let workspace = state[monitorIdx]?.workspaces[workspaceIdx];
      if (workspace) {
        console.log('????');
        workspace.layoutRules = null;
      }
    },
    enableCustomLayoutRules: (state, action: PayloadAction<ForWorkspace>) => {
      const { workspaceIdx, monitorIdx } = action.payload;
      let workspace = state[monitorIdx]?.workspaces[workspaceIdx];
      if (!workspace) {
        return;
      }
      workspace.customLayoutRules = {};
      for (let n = 1; n < 10; n++) {
        workspace.customLayoutRules[n] = workspace.customLayout;
      }
    },
    disableCustomLayoutRules: (state, action: PayloadAction<ForWorkspace>) => {
      const { workspaceIdx, monitorIdx } = action.payload;
      let workspace = state[monitorIdx]?.workspaces[workspaceIdx];
      if (workspace) {
        workspace.customLayoutRules = null;
        console.log('????');
      }
    },
  },
});

export const MonitorsActions = MonitorsSlice.actions;