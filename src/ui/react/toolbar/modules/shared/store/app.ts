import { createSlice, type Dispatch } from "@reduxjs/toolkit";
import { BluetoothDevices, invoke, LanguageList, SeelenCommand, UserDetails } from "@seelen-ui/lib";
import { StateBuilder } from "libs/ui/react/utils/StateBuilder.ts";

import type { RootState } from "./domain.ts";
import { PowerMode } from "node_modules/@seelen-ui/lib/esm/gen/types/PowerMode";

const initialState: RootState = {
  version: 0,
  user: (await UserDetails.getAsync()).user,
  env: (await invoke(SeelenCommand.GetUserEnvs)) as Record<string, string>,
  bluetoothDevices: (await BluetoothDevices.getAsync()).all(),
  // default values of https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-system_power_status
  powerStatus: {
    acLineStatus: 255,
    batteryFlag: 255,
    batteryLifePercent: 255,
    systemStatusFlag: 0,
    batteryLifeTime: -1,
    batteryFullLifeTime: -1,
  },
  powerPlan: PowerMode.Unknown,
  batteries: [],
  networkAdapters: [],
  networkLocalIp: null,
  online: false,
  wlanBssEntries: [],
  mediaSessions: [],
  mediaOutputs: [],
  mediaInputs: [],
  notifications: [],
  languages: [],
};

export const RootSlice = createSlice({
  name: "root",
  initialState,
  reducers: {
    ...StateBuilder.reducersFor(initialState),
  },
});

export const RootActions = RootSlice.actions;
export const Selectors = StateBuilder.compositeSelector(initialState);

// no core things that can be lazy loaded to improve performance
export function lazySlice(d: Dispatch) {
  invoke(SeelenCommand.GetNotifications).then((notifications) => d(RootActions.setNotifications(notifications)));

  invoke(SeelenCommand.GetPowerStatus).then((status) => d(RootActions.setPowerStatus(status)));
  invoke(SeelenCommand.GetPowerMode).then((plan) => d(RootActions.setPowerPlan(plan)));
  invoke(SeelenCommand.GetBatteries).then((batteries) => d(RootActions.setBatteries(batteries)));

  invoke(SeelenCommand.GetMediaDevices).then(([inputs, outputs]) => {
    d(RootActions.setMediaInputs(inputs));
    d(RootActions.setMediaOutputs(outputs));
  });

  invoke(SeelenCommand.GetMediaSessions).then((sessions) => d(RootActions.setMediaSessions(sessions)));

  invoke(SeelenCommand.GetNetworkDefaultLocalIp).then((ip) => d(RootActions.setNetworkLocalIp(ip)));
  invoke(SeelenCommand.GetNetworkAdapters).then((adapters) => d(RootActions.setNetworkAdapters(adapters)));
  invoke(SeelenCommand.GetNetworkInternetConnection).then((online) => d(RootActions.setOnline(online)));

  LanguageList.getAsync().then((list) => d(RootActions.setLanguages(list.asArray())));
}
