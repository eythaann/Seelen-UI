import { createSelector, createSlice, PayloadAction } from '@reduxjs/toolkit';
import { DocumentsFolder, DownloadsFolder, MusicFolder, PicturesFolder, RecentFolder, Settings, UIColors, VideosFolder } from '@seelen-ui/lib';
import { Placeholder, ToolbarItem } from '@seelen-ui/lib/types';

import { RootState } from './domain';

import { StateBuilder } from '../../../../shared/StateBuilder';

const settings = await Settings.default();

const initialState: RootState = {
  version: 0,
  placeholder: null,
  plugins: [],
  dateFormat: '',
  isOverlaped: false,
  user: null,
  userRecentFolder: RecentFolder.default().all(),
  userDocumentsFolder: DocumentsFolder.default().all(),
  userDownloadsFolder: DownloadsFolder.default().all(),
  userPicturesFolder: PicturesFolder.default().all(),
  userVideosFolder: VideosFolder.default().all(),
  userMusicFolder: MusicFolder.default().all(),
  focused: null,
  settings: settings.inner.fancyToolbar,
  env: {},
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
  colors: UIColors.default().inner,
};

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
    setPlaceholder(state, action: PayloadAction<Placeholder | null>) {
      state.placeholder = action.payload;
      state.version++;
    },
    setItemsOnLeft(state, action: PayloadAction<ToolbarItem[]>) {
      if (state.placeholder) {
        state.placeholder.left = action.payload;
      }
    },
    setItemsOnCenter(state, action: PayloadAction<ToolbarItem[]>) {
      if (state.placeholder) {
        state.placeholder.center = action.payload;
      }
    },
    setItemsOnRight(state, action: PayloadAction<ToolbarItem[]>) {
      if (state.placeholder) {
        state.placeholder.right = action.payload;
      }
    },
    addItem(state, action: PayloadAction<string>) {
      if (!state.placeholder) {
        return;
      }
      const alreadyExists =
        state.placeholder.left.includes(action.payload) ||
        state.placeholder.right.includes(action.payload) ||
        state.placeholder.center.includes(action.payload);
      if (!alreadyExists) {
        state.placeholder.right.push(action.payload);
      }
    },
    removeItem(state, action: PayloadAction<string>) {
      let id = action.payload;
      if (state.placeholder) {
        let filter = (d: any) => d !== id && d.id !== id;
        state.placeholder.left = state.placeholder.left.filter(filter);
        state.placeholder.center = state.placeholder.center.filter(filter);
        state.placeholder.right = state.placeholder.right.filter(filter);
      }
    },
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);

export const selectDefaultOutput = createSelector(Selectors.mediaOutputs, (mediaOutputs) =>
  mediaOutputs.find((d) => d.is_default_multimedia),
);
