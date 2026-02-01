import { useComputed } from "@preact/signals";
import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { ToolbarJsScope } from "@seelen-ui/lib/types";
import type { WidgetId } from "@seelen-ui/lib/types";
import { useSyncClockInterval, useThrottle } from "libs/ui/react/utils/hooks";
import moment from "moment";
import { useEffect, useState } from "preact/hooks";
import { useTranslation } from "react-i18next";
import { $allByWidget, $settings } from "./mod";
import { $virtual_desktop } from "./system";
import { $focused } from "./windows";
import * as globalState from "./global";

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
  const count = useComputed(() => globalState.$notifications.value.length);
  return {
    count: count.value,
  };
}

function useMediaScope() {
  const defaultOutputDevice = useComputed(() => globalState.$media_outputs.value.find((d) => d.isDefaultMultimedia));
  const defaultInputDevice = useComputed(() => globalState.$media_inputs.value.find((d) => d.isDefaultMultimedia));
  const defaultMediaSession = useComputed(() => globalState.$media_sessions.value.find((d) => d.default));

  const { id, volume = 0, muted: isMuted = true } = defaultOutputDevice.value || {};

  const { volume: inputVolume = 0, muted: inputIsMuted = true } = defaultInputDevice.value || {};

  const mediaSession = defaultMediaSession.value || null;

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
  const online = useComputed(() => globalState.$online.value);
  const interfaces = useComputed(() => globalState.$network_adapters.value);
  const defaultIp = useComputed(() => globalState.$network_local_ip.value);

  const usingInterface = useComputed(
    () => interfaces.value.find((i) => i.ipv4 === defaultIp.value) || null,
  );

  return {
    online: online.value,
    interfaces: interfaces.value,
    usingInterface: usingInterface.value,
  };
}

function useKeyboardScope() {
  const languages = useComputed(() => globalState.$languages.value);
  const activeLang = useComputed(
    () => languages.value.find((l) => l.keyboardLayouts.some((k) => k.active)) || languages.value[0],
  );
  const activeKeyboard = useComputed(
    () =>
      activeLang.value?.keyboardLayouts.find((k) => k.active) ||
      activeLang.value?.keyboardLayouts[0],
  );

  let activeLangPrefix = activeLang.value?.nativeName
    .split("")
    .slice(0, 3)
    .filter((c) => !["(", ")", " "].includes(c))
    .join("")
    .toLocaleUpperCase() || "";

  let words = activeKeyboard.value?.displayName.split(/[\s\-\(\)]/) || [];
  let activeKeyboardPrefix = words.length > 1
    ? words
      .map((word) => word[0])
      .join("")
      .toLocaleUpperCase()
    : words[0]?.slice(0, 3).toLocaleUpperCase() || "";

  return {
    activeLang: activeLang.value,
    activeKeyboard: activeKeyboard.value,
    activeLangPrefix,
    activeKeyboardPrefix,
    languages: languages.value,
  };
}

function useUserScope() {
  const user = useComputed(() => globalState.$user.value);
  const userMenuConfig = useComputed(
    () => $allByWidget.value?.["@seelen/user-menu" as WidgetId],
  );

  const displayName = useComputed(() => {
    const source = (userMenuConfig.value as Record<string, unknown> | undefined)?.displayNameSource;
    if (source === "xboxGamertag" && user.value?.xboxGamertag) {
      return user.value.xboxGamertag;
    }
    return user.value?.name;
  });

  return {
    user: { ...user.value, displayName: displayName.value },
  };
}

function useBluetoothScope() {
  const bluetoothDevices = useComputed(() => globalState.$bluetooth_devices.value);
  const connectedDevices = useComputed(() => bluetoothDevices.value.filter((item) => item.connected));

  return {
    devices: bluetoothDevices.value,
    connectedDevices: connectedDevices.value,
  };
}

function usePowerScope() {
  const power = useComputed(() => globalState.$power_status.value);
  const powerMode = useComputed(() => globalState.$power_plan.value);
  const batteries = useComputed(() => globalState.$batteries.value);

  return {
    power: power.value,
    powerMode: powerMode.value,
    batteries: batteries.value,
  };
}

function useFocusedAppScope() {
  return {
    focusedApp: $focused.value,
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
