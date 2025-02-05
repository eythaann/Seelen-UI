import { createSlice, Dispatch, PayloadAction } from '@reduxjs/toolkit';
import {
  ApplicationHistory,
  DesktopFolder,
  DocumentsFolder,
  DownloadsFolder,
  invoke,
  MusicFolder,
  PicturesFolder,
  RecentFolder,
  SeelenCommand,
  Settings,
  UIColors,
  UserDetails,
  VideosFolder,
} from '@seelen-ui/lib';
import { Placeholder, ToolbarItem } from '@seelen-ui/lib/types';

import { RootState } from './domain';

import { StateBuilder } from '../../../../shared/StateBuilder';

const settings = await Settings.default();

const initialState: RootState = {
  version: 0,
  items: await invoke(SeelenCommand.StateGetToolbarItems),
  plugins: [],
  dateFormat: '',
  isOverlaped: false,
  user: (await UserDetails.getAsync()).user,
  userRecentFolder: [],
  userDesktopFolder: [],
  userDocumentsFolder: [],
  userDownloadsFolder: [],
  userPicturesFolder: [],
  userVideosFolder: [],
  userMusicFolder: [],
  focused: null,
  settings: settings.fancyToolbar,
  env: (await invoke(SeelenCommand.GetUserEnvs)) as Record<string, string>,
  // default values of https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-system_power_status
  powerStatus: {
    acLineStatus: 255,
    batteryFlag: 255,
    batteryLifePercent: 255,
    systemStatusFlag: 0,
    batteryLifeTime: -1,
    batteryFullLifeTime: -1,
  },
  batteries: [],
  workspaces: [],
  activeWorkspace: null,
  systemTray: [],
  networkAdapters: [],
  networkLocalIp: null,
  online: false,
  wlanBssEntries: [],
  mediaSessions: [],
  mediaOutputs: [],
  mediaInputs: [],
  notifications: [],
  history: [],
  historyOnMonitor: [],
  colors: UIColors.default().inner,
};

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
    setPlaceholder(state, action: PayloadAction<Placeholder>) {
      state.items = action.payload;
      state.version++;
    },
    setItemsOnLeft(state, action: PayloadAction<ToolbarItem[]>) {
      if (state.items) {
        state.items.left = action.payload;
      }
    },
    setItemsOnCenter(state, action: PayloadAction<ToolbarItem[]>) {
      if (state.items) {
        state.items.center = action.payload;
      }
    },
    setItemsOnRight(state, action: PayloadAction<ToolbarItem[]>) {
      if (state.items) {
        state.items.right = action.payload;
      }
    },
    addItem(state, action: PayloadAction<string>) {
      if (!state.items) {
        return;
      }
      const alreadyExists =
        state.items.left.includes(action.payload) ||
        state.items.right.includes(action.payload) ||
        state.items.center.includes(action.payload);
      if (!alreadyExists) {
        state.items.right.push(action.payload);
      }
    },
    removeItem(state, action: PayloadAction<string>) {
      let id = action.payload;
      if (state.items) {
        let filter = (d: any) => d !== id && d.id !== id;
        state.items.left = state.items.left.filter(filter);
        state.items.center = state.items.center.filter(filter);
        state.items.right = state.items.right.filter(filter);
      }
    },
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);

// no core things that can be lazy loaded to improve performance
export async function lazySlice(d: Dispatch) {
  const obj = {
    userRecentFolder: (await RecentFolder.getAsync()).asArray(),
    userDesktopFolder: (await DesktopFolder.getAsync()).asArray(),
    userDocumentsFolder: (await DocumentsFolder.getAsync()).asArray(),
    userDownloadsFolder: (await DownloadsFolder.getAsync()).asArray(),
    userPicturesFolder: (await PicturesFolder.getAsync()).asArray(),
    userVideosFolder: (await VideosFolder.getAsync()).asArray(),
    userMusicFolder: (await MusicFolder.getAsync()).asArray(),
    history: (await ApplicationHistory.getAsync()).asArray(),
    historyOnMonitor: (await ApplicationHistory.getCurrentMonitorHistoryAsync()).asArray(),
  };
  d(RootActions.setUserRecentFolder(obj.userRecentFolder));
  d(RootActions.setUserDesktopFolder(obj.userDesktopFolder));
  d(RootActions.setUserDocumentsFolder(obj.userDocumentsFolder));
  d(RootActions.setUserDownloadsFolder(obj.userDownloadsFolder));
  d(RootActions.setUserPicturesFolder(obj.userPicturesFolder));
  d(RootActions.setUserVideosFolder(obj.userVideosFolder));
  d(RootActions.setUserMusicFolder(obj.userMusicFolder));
  d(RootActions.setHistory(obj.history));
  d(RootActions.setHistoryOnMonitor(obj.historyOnMonitor));
}
