import { useComputed } from "@preact/signals";
import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { ToolbarJsScope } from "@seelen-ui/lib/types";
import type { WidgetId } from "@seelen-ui/lib/types";
import { useSyncClockInterval, useThrottle } from "libs/ui/react/utils/hooks";
import moment from "moment";
import { useEffect, useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { $allByWidget, $settings } from "../../../shared/state/mod";
import { $virtual_desktop } from "../../../shared/state/system";
import { $focused } from "../../../shared/state/windows";
import {
  useLazyBatteries,
  useLazyBluetoothDevices,
  useLazyCores,
  useLazyDisks,
  useLazyLanguages,
  useLazyMediaDevices,
  useLazyMediaSessions,
  useLazyMemory,
  useLazyNetworkAdapters,
  useLazyNetworkLocalIp,
  useLazyNetworkStatistics,
  useLazyNotifications,
  useLazyOnline,
  useLazyPowerMode,
  useLazyPowerStatus,
  useLazyUser,
} from "../../../shared/state/lazy";

export function useItemScope(scopes: Readonly<ToolbarJsScope[]>) {
  let fetching = false;
  const scope = {} as Record<any, any>;

  if (scopes.includes(ToolbarJsScope.Date)) {
    const dateScope = useDateScope();
    if (dateScope.fetching) fetching = true;
    if (dateScope.data) Object.assign(scope, dateScope.data);
  }

  if (scopes.includes(ToolbarJsScope.Notifications)) {
    const notificationsScope = useNotificationsScope();
    if (notificationsScope.fetching) fetching = true;
    if (notificationsScope.data) Object.assign(scope, notificationsScope.data);
  }

  if (scopes.includes(ToolbarJsScope.Media)) {
    const mediaScope = useMediaScope();
    if (mediaScope.fetching) fetching = true;
    if (mediaScope.data) Object.assign(scope, mediaScope.data);
  }

  if (scopes.includes(ToolbarJsScope.Network)) {
    const networkScope = useNetworkScope();
    if (networkScope.fetching) fetching = true;
    if (networkScope.data) Object.assign(scope, networkScope.data);
  }

  if (scopes.includes(ToolbarJsScope.Keyboard)) {
    const keyboardScope = useKeyboardScope();
    if (keyboardScope.fetching) fetching = true;
    if (keyboardScope.data) Object.assign(scope, keyboardScope.data);
  }

  if (scopes.includes(ToolbarJsScope.User)) {
    const userScope = useUserScope();
    if (userScope.fetching) fetching = true;
    if (userScope.data) Object.assign(scope, userScope.data);
  }

  if (scopes.includes(ToolbarJsScope.Bluetooth)) {
    const bluetoothScope = useBluetoothScope();
    if (bluetoothScope.fetching) fetching = true;
    if (bluetoothScope.data) Object.assign(scope, bluetoothScope.data);
  }

  if (scopes.includes(ToolbarJsScope.Power)) {
    const powerScope = usePowerScope();
    if (powerScope.fetching) fetching = true;
    if (powerScope.data) Object.assign(scope, powerScope.data);
  }

  if (scopes.includes(ToolbarJsScope.FocusedApp)) {
    const focusedAppScope = useFocusedAppScope();
    if (focusedAppScope.fetching) fetching = true;
    if (focusedAppScope.data) Object.assign(scope, focusedAppScope.data);
  }

  if (scopes.includes(ToolbarJsScope.Workspaces)) {
    const workspacesScope = useWorkspacesScope();
    if (workspacesScope.fetching) fetching = true;
    if (workspacesScope.data) Object.assign(scope, workspacesScope.data);
  }

  if (scopes.includes(ToolbarJsScope.Disk)) {
    const diskScope = useDiskScope();
    if (diskScope.fetching) fetching = true;
    if (diskScope.data) Object.assign(scope, diskScope.data);
  }

  if (scopes.includes(ToolbarJsScope.NetworkStatistics)) {
    const networkStatisticsScope = useNetworkStatisticsScope();
    if (networkStatisticsScope.fetching) fetching = true;
    if (networkStatisticsScope.data) Object.assign(scope, networkStatisticsScope.data);
  }

  if (scopes.includes(ToolbarJsScope.Memory)) {
    const memoryScope = useMemoryScope();
    if (memoryScope.fetching) fetching = true;
    if (memoryScope.data) Object.assign(scope, memoryScope.data);
  }

  if (scopes.includes(ToolbarJsScope.Cpu)) {
    const cpuScope = useCpuScope();
    if (cpuScope.fetching) fetching = true;
    if (cpuScope.data) Object.assign(scope, cpuScope.data);
  }

  return { fetching, data: scope };
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
    fetching: false,
    data: { date },
  };
}

function useNotificationsScope() {
  const { fetching, data: notifications } = useLazyNotifications();

  if (fetching) {
    return { fetching: true, data: null };
  }

  return {
    fetching: false,
    data: { count: notifications?.length || 0 },
  };
}

function useMediaScope() {
  const { fetching: fetchingDevices, data: mediaDevices } = useLazyMediaDevices();
  const { fetching: fetchingSessions, data: mediaSessions } = useLazyMediaSessions();

  if (fetchingDevices || fetchingSessions) {
    return { fetching: true, data: null };
  }

  const [mediaInputs, mediaOutputs] = mediaDevices!;

  const defaultOutputDevice = mediaOutputs.find((d: any) => d.isDefaultMultimedia);
  const defaultInputDevice = mediaInputs.find((d: any) => d.isDefaultMultimedia);
  const defaultMediaSession = mediaSessions!.find((d: any) => d.default);

  const { id, volume = 0, muted: isMuted = true } = defaultOutputDevice || {};

  const { volume: inputVolume = 0, muted: inputIsMuted = true } = defaultInputDevice || {};

  const mediaSession = defaultMediaSession || null;

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
    fetching: false,
    data: {
      volume,
      isMuted,
      inputVolume,
      inputIsMuted,
      mediaSession,
      onWheel,
    },
  };
}

function useNetworkScope() {
  const { fetching: fetchingOnline, data: online } = useLazyOnline();
  const { fetching: fetchingInterfaces, data: interfaces } = useLazyNetworkAdapters();
  const { fetching: fetchingIp, data: defaultIp } = useLazyNetworkLocalIp();

  if (fetchingOnline || fetchingInterfaces || fetchingIp) {
    return { fetching: true, data: null };
  }

  const usingInterface = interfaces?.find((i: any) => i.ipv4 === defaultIp) || null;

  return {
    fetching: false,
    data: {
      online,
      interfaces,
      usingInterface,
    },
  };
}

function useKeyboardScope() {
  const { fetching, data: languages } = useLazyLanguages();

  if (fetching) {
    return { fetching: true, data: null };
  }

  const activeLang = languages?.find((l: any) => l.keyboardLayouts.some((k: any) => k.active)) || languages?.[0];
  const activeKeyboard = activeLang?.keyboardLayouts.find((k: any) => k.active) || activeLang?.keyboardLayouts[0];

  let activeLangPrefix = activeLang?.nativeName
    ?.split("")
    .slice(0, 3)
    .filter((c: any) => !["(", ")", " "].includes(c))
    .join("")
    .toLocaleUpperCase() || "";

  let words = activeKeyboard?.displayName?.split(/[\s\-\(\)]/) || [];
  let activeKeyboardPrefix = words.length > 1
    ? words
      .map((word: any) => word[0])
      .join("")
      .toLocaleUpperCase()
    : words[0]?.slice(0, 3).toLocaleUpperCase() || "";

  return {
    fetching: false,
    data: {
      activeLang,
      activeKeyboard,
      activeLangPrefix,
      activeKeyboardPrefix,
      languages,
    },
  };
}

function useUserScope() {
  const { fetching, data: user } = useLazyUser();

  if (fetching) {
    return { fetching: true, data: null };
  }

  const userMenuConfig = $allByWidget.value?.["@seelen/user-menu" as WidgetId];
  const source = (userMenuConfig as Record<string, unknown> | undefined)?.displayNameSource;
  const displayName = source === "xboxGamertag" && user?.xboxGamertag ? user.xboxGamertag : user?.name;

  return {
    fetching: false,
    data: {
      user: { ...user, displayName },
    },
  };
}

function useBluetoothScope() {
  const { fetching, data: bluetoothDevices } = useLazyBluetoothDevices();

  if (fetching) {
    return { fetching: true, data: null };
  }

  const connectedDevices = bluetoothDevices?.filter((item: any) => item.connected) || [];

  return {
    fetching: false,
    data: {
      devices: bluetoothDevices,
      connectedDevices,
    },
  };
}

function usePowerScope() {
  const { fetching: fetchingPower, data: power } = useLazyPowerStatus();
  const { fetching: fetchingMode, data: powerMode } = useLazyPowerMode();
  const { fetching: fetchingBatteries, data: batteries } = useLazyBatteries();

  if (fetchingPower || fetchingMode || fetchingBatteries) {
    return { fetching: true, data: null };
  }

  return {
    fetching: false,
    data: {
      power,
      powerMode,
      batteries,
    },
  };
}

function useFocusedAppScope() {
  return {
    fetching: false,
    data: {
      focusedApp: $focused.value,
    },
  };
}

function useWorkspacesScope() {
  const workspaces = $virtual_desktop.value?.workspaces || [];
  const activeWorkspace = $virtual_desktop.value?.active_workspace;

  const onWheel = useThrottle(
    (isUp: boolean) => {
      const index = workspaces.findIndex((w: any) => w.id === activeWorkspace);
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
    fetching: false,
    data: {
      workspaces,
      activeWorkspace,
      onWheel,
    },
  };
}

function useDiskScope() {
  const { fetching, data: disks } = useLazyDisks();

  if (fetching) {
    return { fetching: true, data: null };
  }

  return {
    fetching: false,
    data: { disks },
  };
}

function useNetworkStatisticsScope() {
  const { fetching, data: networkStatistics } = useLazyNetworkStatistics();

  if (fetching) {
    return { fetching: true, data: null };
  }

  return {
    fetching: false,
    data: { networkStatistics },
  };
}

function useMemoryScope() {
  const { fetching, data: memory } = useLazyMemory();

  if (fetching) {
    return { fetching: true, data: null };
  }

  return {
    fetching: false,
    data: { memory },
  };
}

function useCpuScope() {
  const { fetching, data: cores } = useLazyCores();

  if (fetching) {
    return { fetching: true, data: null };
  }

  return {
    fetching: false,
    data: { cores },
  };
}
