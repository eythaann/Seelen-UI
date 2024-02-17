import { createSlice } from '@reduxjs/toolkit';

import { Layout } from '../layouts/domain';
import { Monitor } from './domain';

const initialState: Monitor[] = [{
  edditingWorkspace: 0,
  workAreaOffset: null,
  workspaces: [{
    name: 'Workspace 1',
    layout: Layout.BSP,
    layout_rules: null,
    containerPadding: null,
    workspacePadding: null,
    custom_layout: null,
    custom_layout_rules: null,
  }],
},
{
  edditingWorkspace: 0,
  workAreaOffset: null,
  workspaces: [{
    name: 'Workspace 1',
    layout: Layout.COLUMNS,
    layout_rules: null,
    containerPadding: null,
    workspacePadding: null,
    custom_layout: null,
    custom_layout_rules: null,
  }],
},
{
  edditingWorkspace: 0,
  workAreaOffset: null,
  workspaces: [{
    name: 'Workspace 1',
    layout: Layout.ROWS,
    layout_rules: null,
    containerPadding: null,
    workspacePadding: null,
    custom_layout: null,
    custom_layout_rules: null,
  }],
},
{
  edditingWorkspace: 0,
  workAreaOffset: null,
  workspaces: [{
    name: 'Workspace 1',
    layout: Layout.HORIZONTAL_STACK,
    layout_rules: null,
    containerPadding: null,
    workspacePadding: null,
    custom_layout: null,
    custom_layout_rules: null,
  }],
},
{
  edditingWorkspace: 0,
  workAreaOffset: null,
  workspaces: [{
    name: 'Workspace 1',
    layout: Layout.VERTICAL_STACK,
    layout_rules: null,
    containerPadding: null,
    workspacePadding: null,
    custom_layout: null,
    custom_layout_rules: null,
  }],
},
{
  edditingWorkspace: 0,
  workAreaOffset: null,
  workspaces: [{
    name: 'Workspace 1',
    layout: Layout.ULTRAWIDE_VERTICAL_STACK,
    layout_rules: null,
    containerPadding: null,
    workspacePadding: null,
    custom_layout: null,
    custom_layout_rules: null,
  }],
}];

export const MonitorsSlice = createSlice({
  name: 'monitors',
  initialState,
  reducers: {

  },
});

export const MonitorsActions = MonitorsSlice.actions;