// todo remove all anys from this file, should be done with the remotion of redux state.

import { useComputed } from "@preact/signals";
import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { ToolbarJsScope } from "@seelen-ui/lib/types";
import { useSyncClockInterval, useThrottle } from "libs/ui/react/utils/hooks";
import moment from "moment";
import { useEffect, useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { useSelector } from "react-redux";
import { Selectors } from "../store/app";
import type { RootState } from "../store/domain";
import { $settings } from "./mod";
import { $virtual_desktop } from "./system";
import { $focused } from "./windows";

export function useItemScope(scopes: Readonly<ToolbarJsScope[]>) {
  const scope = {} as Record<any, any>;

  if (scopes.includes(ToolbarJsScope.Date)) {
    Object.assign(scope, useDateScope());
  }

  if (scopes.includes(ToolbarJsScope.Notifications)) {
    Object.assign(scope, useNotificationsScope());
  }

  if (scopes.includes(ToolbarJsScope.Media)) {
    Object.assign(scope, useMediaScope());
  }

  if (scopes.includes(ToolbarJsScope.Network)) {
    Object.assign(scope, useNetworkScope());
  }

  if (scopes.includes(ToolbarJsScope.Keyboard)) {
    Object.assign(scope, useKeyboardScope());
  }

  if (scopes.includes(ToolbarJsScope.User)) {
    Object.assign(scope, useUserScope());
  }

  if (scopes.includes(ToolbarJsScope.Bluetooth)) {
    Object.assign(scope, useBluetoothScope());
  }

  if (scopes.includes(ToolbarJsScope.Power)) {
    Object.assign(scope, usePowerScope());
  }

  if (scopes.includes(ToolbarJsScope.FocusedApp)) {
    Object.assign(scope, useFocusedAppScope());
  }

  if (scopes.includes(ToolbarJsScope.Workspaces)) {
    Object.assign(scope, useWorkspacesScope());
  }

  return scope;
}

function useDateScope() {
  const momentJsLangMap: { [key: string]: string } = {
    no: "nb",
    zh: "zh-cn",
  };

  const $date_format = useComputed(() => $settings.value.dateFormat);

  const {
    i18n: { language: lang },
  } = useTranslation();
  let language = momentJsLangMap[lang] || lang;

  const [date, setDate] = useState(moment().locale(language).format($date_format.value));

  // inmediately update the date, like interval is reseted on deps change
  useEffect(() => {
    setDate(moment().locale(language).format($date_format.value));
  }, [$date_format.value, language]);

  useSyncClockInterval(
    () => {
      setDate(moment().locale(language).format($date_format.value));
    },
    $date_format.value.includes("ss") ? "seconds" : "minutes",
    [$date_format.value, language],
  );

  return {
    date,
  };
}

function useNotificationsScope() {
  const count = useSelector((state: RootState) => Selectors.notifications(state).length);

  return {
    count,
  };
}

function useMediaScope() {
  const {
    id,
    volume = 0,
    muted: isMuted = true,
  } = useSelector((state: RootState) => Selectors.mediaOutputs(state).find((d: any) => d.isDefaultMultimedia)) || {};

  const { volume: inputVolume = 0, muted: inputIsMuted = true } =
    useSelector((state: RootState) => Selectors.mediaInputs(state).find((d: any) => d.isDefaultMultimedia)) || {};

  const mediaSession = useSelector((state: RootState) => Selectors.mediaSessions(state).find((d: any) => d.default)) ||
    null;

  function onWheel(e: WheelEvent) {
    const isUp = e.deltaY < 0;
    const level = Math.max(0, Math.min(1, volume + (isUp ? 0.02 : -0.02)));
    if (id) {
      invoke(SeelenCommand.SetVolumeLevel, {
        deviceId: id,
        level,
        sessionId: null,
      });
    }
  }

  return {
    volume,
    isMuted,
    inputVolume,
    inputIsMuted,
    mediaSession,
    onWheel,
  };
}

function useNetworkScope() {
  const networkAdapters: any = useSelector(Selectors.networkAdapters);
  const defaultIp = useSelector(Selectors.networkLocalIp);
  const online = useSelector(Selectors.online);

  const usingAdapter = networkAdapters.find((i: any) => i.ipv4 === defaultIp) || null;

  return {
    online,
    interfaces: networkAdapters,
    usingInterface: usingAdapter,
  };
}

function useKeyboardScope() {
  const languages: any = useSelector(Selectors.languages);

  const activeLang = languages.find((l: any) => l.keyboardLayouts.some((k: any) => k.active)) || languages[0];
  const activeKeyboard = activeLang?.keyboardLayouts.find((k: any) => k.active) || activeLang?.keyboardLayouts[0];

  if (!activeLang || !activeKeyboard) {
    console.error("No active keyboard for unknown reason");
    return {
      activeLang: null,
      activeKeyboard: null,
      activeLangPrefix: "",
      activeKeyboardPrefix: "",
      languages,
    };
  }

  let activeLangPrefix = activeLang.nativeName
    .split("")
    .slice(0, 3)
    .filter((c: any) => !["(", ")", " "].includes(c))
    .join("")
    .toLocaleUpperCase();

  let words = activeKeyboard.displayName.split(/[\s\-\(\)]/);
  let activeKeyboardPrefix = words.length > 1
    ? words
      .map((word: any) => word[0])
      .join("")
      .toLocaleUpperCase()
    : words[0]?.slice(0, 3).toLocaleUpperCase() || "";

  return {
    activeLang,
    activeKeyboard,
    activeLangPrefix,
    activeKeyboardPrefix,
    languages,
  };
}

function useUserScope() {
  const user = useSelector(Selectors.user);

  return {
    user,
  };
}

function useBluetoothScope() {
  const bluetoothDevices: any = useSelector(Selectors.bluetoothDevices);
  const connectedDevices = bluetoothDevices.filter((item: any) => item.connected);

  return {
    devices: bluetoothDevices,
    connectedDevices,
  };
}

function usePowerScope() {
  const power = useSelector(Selectors.powerStatus);
  const powerMode = useSelector(Selectors.powerPlan);
  const batteries = useSelector(Selectors.batteries);

  return {
    power,
    powerMode,
    batteries,
  };
}

function useFocusedAppScope() {
  const focusedApp = $focused.value;
  return {
    focusedApp,
  };
}

function useWorkspacesScope() {
  const workspaces = $virtual_desktop.value?.workspaces || [];
  const activeWorkspace = $virtual_desktop.value?.active_workspace;

  const onWheel = useThrottle(
    (isUp: boolean) => {
      const index = workspaces.findIndex((w) => w.id === activeWorkspace);
      const newIndex = isUp ? index - 1 : index + 1;
      if (newIndex >= 0 && newIndex < workspaces.length) {
        let workspace = workspaces[newIndex]!;
        invoke(SeelenCommand.SwitchWorkspace, { workspaceId: workspace.id });
      }
    },
    500,
    { trailing: false },
  );

  return {
    workspaces,
    activeWorkspace,
    onWheel,
  };
}
