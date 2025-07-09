import { createSlice, Dispatch, PayloadAction } from '@reduxjs/toolkit';
import {
  BluetoothDevices,
  DesktopFolder,
  DocumentsFolder,
  DownloadsFolder,
  invoke,
  LanguageList,
  MusicFolder,
  PicturesFolder,
  RecentFolder,
  SeelenCommand,
  UserDetails,
  VideosFolder,
  WegItems,
} from '@seelen-ui/lib';

import { RootState } from './domain';

import { StateBuilder } from '../../../../shared/StateBuilder';

const initialState: RootState = {
  version: 0,
  user: (await UserDetails.getAsync()).user,
  userRecentFolder: [],
  userDesktopFolder: [],
  userDocumentsFolder: [],
  userDownloadsFolder: [],
  userPicturesFolder: [],
  userVideosFolder: [],
  userMusicFolder: [],
  focused: null,
  env: (await invoke(SeelenCommand.GetUserEnvs)) as Record<string, string>,
  bluetoothDevices: (await BluetoothDevices.getAsync()).all(),
  discoveredBluetoothDevices: BluetoothDevices.default().all(),
  // default values of https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-system_power_status
  powerStatus: {
    acLineStatus: 255,
    batteryFlag: 255,
    batteryLifePercent: 255,
    systemStatusFlag: 0,
    batteryLifeTime: -1,
    batteryFullLifeTime: -1,
  },
  powerPlan: 'Unknown',
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
  languages: [],
  openApps: [],
  windowColorByHandle: {},
};

export const RootSlice = createSlice({
  name: 'root',
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
    addWindowColor(
      state,
      action: PayloadAction<[number, { background: string; foreground: string }]>,
    ) {
      state.windowColorByHandle[`${action.payload[0]}`] = action.payload[1];
    },
    removeWindowColor(state, action: PayloadAction<number>) {
      delete state.windowColorByHandle[`${action.payload}`];
    },
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);

// no core things that can be lazy loaded to improve performance
export async function lazySlice(d: Dispatch) {
  invoke(SeelenCommand.GetNotifications).then((notifications) =>
    d(RootActions.setNotifications(notifications)),
  );

  invoke(SeelenCommand.GetPowerStatus).then((status) => d(RootActions.setPowerStatus(status)));
  invoke(SeelenCommand.GetPowerMode).then((plan) => d(RootActions.setPowerPlan(plan)));
  invoke(SeelenCommand.GetBatteries).then((batteries) => d(RootActions.setBatteries(batteries)));

  invoke(SeelenCommand.GetMediaDevices).then(([inputs, outputs]) => {
    d(RootActions.setMediaInputs(inputs));
    d(RootActions.setMediaOutputs(outputs));
  });

  invoke(SeelenCommand.GetMediaSessions).then((sessions) =>
    d(RootActions.setMediaSessions(sessions)),
  );

  invoke(SeelenCommand.GetTrayIcons).then((info) => d(RootActions.setSystemTray(info)));

  LanguageList.getAsync().then((list) => d(RootActions.setLanguages(list.asArray())));

  const onGetWegItems = (items: WegItems) => {
    const apps = items.inner.left
      .concat(items.inner.center)
      .concat(items.inner.right)
      .map((d) => {
        if ('windows' in d) {
          return d.windows;
        }
        return [];
      })
      .flat()
      .toSorted((a, b) => Number(b.lastActive - a.lastActive));
    d(RootActions.setOpenApps(apps));
  };
  WegItems.forCurrentWidget().then(onGetWegItems);
  WegItems.forCurrentWidgetChange(onGetWegItems);

  const obj = {
    userRecentFolder: (await RecentFolder.getAsync()).asArray(),
    userDesktopFolder: (await DesktopFolder.getAsync()).asArray(),
    userDocumentsFolder: (await DocumentsFolder.getAsync()).asArray(),
    userDownloadsFolder: (await DownloadsFolder.getAsync()).asArray(),
    userPicturesFolder: (await PicturesFolder.getAsync()).asArray(),
    userVideosFolder: (await VideosFolder.getAsync()).asArray(),
    userMusicFolder: (await MusicFolder.getAsync()).asArray(),
  };
  d(RootActions.setUserRecentFolder(obj.userRecentFolder));
  d(RootActions.setUserDesktopFolder(obj.userDesktopFolder));
  d(RootActions.setUserDocumentsFolder(obj.userDocumentsFolder));
  d(RootActions.setUserDownloadsFolder(obj.userDownloadsFolder));
  d(RootActions.setUserPicturesFolder(obj.userPicturesFolder));
  d(RootActions.setUserVideosFolder(obj.userVideosFolder));
  d(RootActions.setUserMusicFolder(obj.userMusicFolder));
}
