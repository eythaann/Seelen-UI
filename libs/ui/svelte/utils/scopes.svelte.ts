// In this files scopes declaration for toolbar and dock plugins are defined

import { type AllSeelenCommandReturns, invoke, SeelenCommand, SeelenEvent, subscribe, Widget } from "@seelen-ui/lib";
import { NotificationsMode } from "@seelen-ui/lib/types";
import { dateState } from "../runes/date.svelte.ts";
import { getIconNameForBTDevice } from "./bluetoothIcons.ts";

class LazyScope<C extends SeelenCommand> {
  private _ready = $state.raw(false);
  private _data = $state.raw<AllSeelenCommandReturns[C] | null>(null);
  private _started = false;

  constructor(
    private command: C,
    private event: SeelenEvent,
  ) {}

  get fetching(): boolean {
    return !this._ready;
  }

  get ready(): boolean {
    return this._ready;
  }

  get data(): AllSeelenCommandReturns[C] | null {
    return this._data;
  }

  lazyInit(): void {
    if (this._started) return;
    this._started = true;

    subscribe(this.event, ({ payload }) => {
      this._data = payload as AllSeelenCommandReturns[C];
      this._ready = true;
    });

    invoke(this.command as any).then((data) => {
      if (!this._ready) {
        this._data = data as AllSeelenCommandReturns[C];
        this._ready = true;
      }
    });
  }
}

const _notifications = new LazyScope(SeelenCommand.GetNotifications, SeelenEvent.Notifications);
const _notificationsMode = new LazyScope(
  SeelenCommand.GetNotificationsMode,
  SeelenEvent.NotificationsModeChanged,
);
const _mediaSessions = new LazyScope(SeelenCommand.GetMediaSessions, SeelenEvent.MediaSessions);
const _mediaDevices = new LazyScope(SeelenCommand.GetMediaDevices, SeelenEvent.MediaDevices);
const _networkOnline = new LazyScope(
  SeelenCommand.GetNetworkInternetConnection,
  SeelenEvent.NetworkInternetConnection,
);
const _networkAdapters = new LazyScope(
  SeelenCommand.GetNetworkAdapters,
  SeelenEvent.NetworkAdapters,
);
const _networkLocalIp = new LazyScope(
  SeelenCommand.GetNetworkDefaultLocalIp,
  SeelenEvent.NetworkDefaultLocalIp,
);
const _languages = new LazyScope(
  SeelenCommand.SystemGetLanguages,
  SeelenEvent.SystemLanguagesChanged,
);
const _imeState = new LazyScope(SeelenCommand.SystemGetImeState, SeelenEvent.SystemImeStateChanged);
const _user = new LazyScope(SeelenCommand.GetUser, SeelenEvent.UserChanged);
const _bluetoothDevices = new LazyScope(
  SeelenCommand.GetBluetoothDevices,
  SeelenEvent.BluetoothDevicesChanged,
);
const _power = new LazyScope(SeelenCommand.GetPowerStatus, SeelenEvent.PowerStatus);
const _powerMode = new LazyScope(SeelenCommand.GetPowerMode, SeelenEvent.PowerMode);
const _batteries = new LazyScope(SeelenCommand.GetBatteries, SeelenEvent.BatteriesStatus);
const _disks = new LazyScope(SeelenCommand.GetSystemDisks, SeelenEvent.SystemDisksChanged);
const _networkStatistics = new LazyScope(
  SeelenCommand.GetSystemNetwork,
  SeelenEvent.SystemNetworkChanged,
);
const _memory = new LazyScope(SeelenCommand.GetSystemMemory, SeelenEvent.SystemMemoryChanged);
const _cores = new LazyScope(SeelenCommand.GetSystemCores, SeelenEvent.SystemCoresChanged);
const _focusedApp = new LazyScope(SeelenCommand.GetFocusedApp, SeelenEvent.GlobalFocusChanged);
const _virtualDesktops = new LazyScope(
  SeelenCommand.StateGetVirtualDesktops,
  SeelenEvent.VirtualDesktopsChanged,
);
const _trayIcons = new LazyScope(SeelenCommand.GetSystemTrayIcons, SeelenEvent.SystemTrayChanged);
const _trashBinInfo = new LazyScope(SeelenCommand.GetTrashBinInfo, SeelenEvent.TrashBinChanged);
const _waveform = new LazyScope(SeelenCommand.GetMediaWaveform, SeelenEvent.MediaWaveform);

type Data = Record<string, unknown>;
export interface ScopesResult {
  fetching: boolean;
  data: Data;
}

export interface ScopeMofidiers {
  userSourceName: string;
}

export function resolveScopes(scopes: string[], { userSourceName }: ScopeMofidiers): ScopesResult {
  const scopesSet = new Set(scopes.map((s) => s.toLowerCase()));

  let fetching = false;
  const data: Data = {};

  if (scopesSet.has("date")) {
    dateStep(data);
  }

  if (scopesSet.has("notifications")) {
    fetching ||= notificationsStep(data);
  }

  if (scopesSet.has("media")) {
    fetching ||= mediaStep(data);
  }

  if (scopesSet.has("network")) {
    fetching ||= networkStep(data);
  }

  if (scopesSet.has("keyboard")) {
    fetching ||= keyboardStep(data);
  }

  if (scopesSet.has("user")) {
    fetching ||= userStep(data, userSourceName);
  }

  if (scopesSet.has("bluetooth")) {
    fetching ||= bluetoothStep(data);
  }

  if (scopesSet.has("power")) {
    fetching ||= powerStep(data);
  }

  if (scopesSet.has("focusedapp")) {
    fetching ||= focusedAppStep(data);
  }

  if (scopesSet.has("workspaces")) {
    fetching ||= workspacesStep(data);
  }

  if (scopesSet.has("disk")) {
    fetching ||= diskStep(data);
  }

  if (scopesSet.has("networkstatistics")) {
    fetching ||= networkStatisticsStep(data);
  }

  if (scopesSet.has("memory")) {
    fetching ||= memoryStep(data);
  }

  if (scopesSet.has("cpu")) {
    fetching ||= cpuStep(data);
  }

  if (scopesSet.has("tray")) {
    fetching ||= trayStep(data);
  }

  if (scopesSet.has("trashbin")) {
    fetching ||= trashBinStep(data);
  }

  if (scopesSet.has("waveform")) {
    fetching ||= waveformStep(data);
  }

  return {
    fetching,
    data,
  };
}

function dateStep(data: Data): void {
  data.date = dateState.formatedDate;
}

function notificationsStep(data: Data): boolean {
  _notifications.lazyInit();
  _notificationsMode.lazyInit();

  if (_notifications.fetching || _notificationsMode.fetching) {
    return true;
  }

  const notifs = _notifications.data;
  data.count = notifs?.length || 0;
  data.dndActive = _notificationsMode.data !== NotificationsMode.All;
  return false;
}

function mediaStep(data: Data): boolean {
  _mediaSessions.lazyInit();
  _mediaDevices.lazyInit();

  if (_mediaSessions.fetching || _mediaDevices.fetching) {
    return true;
  }

  const [mediaInputs, mediaOutputs] = _mediaDevices.data || [[], []];
  const defaultOutputDevice = mediaOutputs.find((d: any) => d.isDefaultMultimedia);
  const defaultInputDevice = mediaInputs.find((d: any) => d.isDefaultMultimedia);
  const defaultMediaSession = _mediaSessions.data?.find((d: any) => d.default);
  const { volume = 0, muted: isMuted = true } = defaultOutputDevice || {};
  const { volume: inputVolume = 0, muted: inputIsMuted = true } = defaultInputDevice || {};

  data.defaultOutputDevice = defaultOutputDevice;
  data.defaultInputDevice = defaultInputDevice;
  data.volume = volume;
  data.isMuted = isMuted;
  data.inputVolume = inputVolume;
  data.inputIsMuted = inputIsMuted;
  data.mediaSession = defaultMediaSession || null;

  return false;
}

function networkStep(data: Data): boolean {
  _networkOnline.lazyInit();
  _networkAdapters.lazyInit();
  _networkLocalIp.lazyInit();

  if (_networkOnline.fetching || _networkAdapters.fetching || _networkLocalIp.fetching) {
    return true;
  }

  const interfaces = _networkAdapters.data;
  const defaultIp = _networkLocalIp.data;
  const usingInterface = interfaces?.find((i: any) => i.ipv4 === defaultIp) || null;
  data.online = _networkOnline.data;
  data.interfaces = interfaces;
  data.usingInterface = usingInterface;

  return false;
}

function keyboardStep(data: Data): boolean {
  _languages.lazyInit();
  _imeState.lazyInit();

  if (_languages.fetching || _imeState.fetching) {
    return true;
  }

  const languages = _languages.data;
  const imeState = _imeState.data;
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

  data.activeLang = activeLang;
  data.activeKeyboard = activeKeyboard;

  data.activeLangPrefix = activeLangPrefix;
  data.activeKeyboardPrefix = activeKeyboardPrefix;

  data.languages = languages;
  data.imeState = imeState;

  return false;
}

function userStep(data: Data, userSourceName: string): boolean {
  _user.lazyInit();

  if (_user.fetching) {
    return true;
  }

  const user = _user.data;
  const displayName = userSourceName === "xboxGamertag" && user?.xboxGamertag ? user.xboxGamertag : user?.name;
  data.user = { ...user, displayName };

  return false;
}

function bluetoothStep(data: Data): boolean {
  _bluetoothDevices.lazyInit();

  if (_bluetoothDevices.fetching) {
    return true;
  }

  data.devices = _bluetoothDevices.data || [];
  data.getIconNameForBTDevice = getIconNameForBTDevice;

  return false;
}

function powerStep(data: Data): boolean {
  _power.lazyInit();
  _powerMode.lazyInit();
  _batteries.lazyInit();

  if (_power.fetching || _powerMode.fetching || _batteries.fetching) {
    return true;
  }

  data.power = _power.data;
  data.powerMode = _powerMode.data;
  data.batteries = _batteries.data;

  return false;
}

function focusedAppStep(data: Data): boolean {
  _focusedApp.lazyInit();

  if (_focusedApp.fetching) {
    return true;
  }

  data.focusedApp = _focusedApp.data;

  return false;
}

function workspacesStep(data: Data): boolean {
  _virtualDesktops.lazyInit();

  if (_virtualDesktops.fetching) {
    return true;
  }

  const monitorId = Widget.getCurrent().decoded.monitorId!;
  const vd = _virtualDesktops.data?.monitors?.[monitorId];
  data.workspaces = vd?.workspaces || [];
  data.activeWorkspace = vd?.active_workspace;

  return false;
}

function diskStep(data: Data): boolean {
  _disks.lazyInit();

  if (_disks.fetching) {
    return true;
  }

  data.disks = _disks.data;

  return false;
}

function networkStatisticsStep(data: Data): boolean {
  _networkStatistics.lazyInit();

  if (_networkStatistics.fetching) {
    return true;
  }

  data.networkStatistics = _networkStatistics.data;

  return false;
}

function memoryStep(data: Data): boolean {
  _memory.lazyInit();

  if (_memory.fetching) {
    return true;
  }

  data.memory = _memory.data;

  return false;
}

function cpuStep(data: Data): boolean {
  _cores.lazyInit();

  if (_cores.fetching) {
    return true;
  }

  data.cores = _cores.data;

  return false;
}

function trayStep(data: Data): boolean {
  _trayIcons.lazyInit();

  if (_trayIcons.fetching) {
    return true;
  }

  data.trayIcons = _trayIcons.data || [];

  return false;
}

function trashBinStep(data: Data): boolean {
  _trashBinInfo.lazyInit();

  if (_trashBinInfo.fetching) {
    return true;
  }

  data.trashBinInfo = _trashBinInfo.data;

  return false;
}

function waveformStep(data: Data): boolean {
  _waveform.lazyInit();

  if (_waveform.fetching) {
    return true;
  }

  data.waveform = _waveform.data;

  return false;
}
