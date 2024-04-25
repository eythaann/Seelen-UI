import { parseAsCamel } from '../../../../utils/schemas';
import { Monitor, MonitorSchema, Workspace, WorkspaceSchema } from '../../../../utils/schemas/Monitors';
import { createSlice, PayloadAction } from '@reduxjs/toolkit';

const initialState: Monitor[] = [parseAsCamel(MonitorSchema, {})];

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
    delete: (state, action: PayloadAction<number>) => {
      state.splice(action.payload, 1);
    },
    insert: (state, action: PayloadAction<number>) => {
      state.splice(action.payload, 0, parseAsCamel(MonitorSchema, {}));
    },
    changeEditingWorkspace: (state, action: PayloadAction<ForWorkspace>) => {
      const { monitorIdx, workspaceIdx } = action.payload;
      const monitor = state[monitorIdx];
      if (!monitor) {
        return;
      }
      monitor.edditingWorkspace = workspaceIdx;
    },
    newWorkspace: (state, action: PayloadAction<ForMonitor & { name: string }>) => {
      const { monitorIdx, name } = action.payload;
      const monitor = state[monitorIdx];
      if (!monitor) {
        return;
      }
      const newWorkspace = parseAsCamel(WorkspaceSchema, {});
      const length = monitor.workspaces.push(newWorkspace);
      newWorkspace.name = name || `Workspace ${length}`;
    },
    updateWorkspace: <T extends keyof Workspace>(state: Monitor[], action: PayloadAction<ForWorkspace & { key: T; value: Workspace[T] }>) => {
      const { workspaceIdx, monitorIdx, key, value } = action.payload;
      let workspace = state[monitorIdx]?.workspaces[workspaceIdx];
      if (!workspace) {
        return;
      }
      workspace[key] = value;
    },
    updateMonitor: <T extends keyof Monitor>(state: Monitor[], action: PayloadAction<ForMonitor & { key: T; value: Monitor[T] }>) => {
      const { monitorIdx, key, value } = action.payload;
      const monitor = state[monitorIdx];
      if (!monitor) {
        return;
      }
      monitor[key] = value;
    },
  },
});

export const MonitorsActions = MonitorsSlice.actions;