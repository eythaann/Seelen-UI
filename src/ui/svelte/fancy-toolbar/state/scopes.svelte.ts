import { type AllSeelenCommandReturns, invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import { NotificationsMode, ToolbarJsScope } from "@seelen-ui/lib/types";
import { throttle } from "lodash";
import moment from "moment";
import { getIconNameForBTDevice } from "libs/ui/svelte/utils/bluetoothIcons";
import { settingsState } from "./settings.svelte.ts";
import { focused, widgetStatuses } from "./windows.svelte.ts";
import { virtualDesktop } from "./system.svelte.ts";

// ── Lazy Scope ────────────────────────────────────────────────────────────────

class LazyScope<C extends SeelenCommand> {
  private _fetching = $state.raw(true);
  private _data = $state.raw<AllSeelenCommandReturns[C] | null>(null);
  private _started = false;

  constructor(
    private command: C,
    private event: SeelenEvent,
  ) {}

  get fetching(): boolean {
    return this._fetching;
  }

  get data(): AllSeelenCommandReturns[C] | null {
    return this._data;
  }

  lazyInit(): void {
    if (this._started) return;
    this._started = true;
    subscribe(this.event, ({ payload }) => {
      this._data = payload as AllSeelenCommandReturns[C];
      this._fetching = false;
    });
    invoke(this.command as any).then((data) => {
      if (this._fetching) {
        this._data = data as AllSeelenCommandReturns[C];
        this._fetching = false;
      }
    });
  }
}

// ── Scopes (lazy — only initialized when first needed) ────────────────────────

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

// ── Date ─────────────────────────────────────────────────────────────────────

const momentJsLangMap: Record<string, string> = { no: "nb" };

let _date = $state.raw("");

$effect.root(() => {
  let timeout: ReturnType<typeof setTimeout> | null = null;
  let interval: ReturnType<typeof setInterval> | null = null;

  $effect(() => {
    const rawLang = settingsState.language;
    const format = settingsState.dateFormat;
    const lang = momentJsLangMap[rawLang] || rawLang.toLowerCase();
    const isSeconds = format.includes("ss");

    if (timeout) clearTimeout(timeout);
    if (interval) clearInterval(interval);

    moment.updateLocale(lang, { postformat: (str: string) => str });
    _date = moment().locale(lang).format(format);

    const now = new Date();
    const msToSync = isSeconds ? 1000 - now.getMilliseconds() : (60 - now.getSeconds()) * 1000 - now.getMilliseconds();

    timeout = setTimeout(() => {
      _date = moment().locale(lang).format(format);
      interval = setInterval(
        () => {
          _date = moment().locale(lang).format(format);
        },
        isSeconds ? 1000 : 60000,
      );
    }, msToSync);

    return () => {
      if (timeout) clearTimeout(timeout);
      if (interval) clearInterval(interval);
    };
  });
});

// ── Workspaces onWheel ───────────────────────────────────────────────────────

const onWheelWorkspace = throttle(
  (isUp: boolean) => {
    const vd = $state.snapshot(virtualDesktop.value);
    const workspaces = vd?.workspaces || [];
    const activeWorkspace = vd?.active_workspace;
    const index = workspaces.findIndex((w: any) => w.id === activeWorkspace);
    const newIndex = isUp ? index - 1 : index + 1;
    if (newIndex >= 0 && newIndex < workspaces.length) {
      invoke(SeelenCommand.SwitchWorkspace, { workspaceId: workspaces[newIndex]!.id });
    }
  },
  500,
  { trailing: false },
);

// ── Scope builder ─────────────────────────────────────────────────────────────

export interface ItemScopeResult {
  fetching: boolean;
  data: Record<string, any>;
}

export function buildItemScope(
  scopes: Readonly<ToolbarJsScope[]>,
  _itemId: string,
  tFn: (key: string) => string,
): ItemScopeResult {
  let fetching = false;
  const data: Record<string, any> = { t: tFn };

  if (scopes.includes(ToolbarJsScope.Date)) {
    data.date = _date;
  }

  if (scopes.includes(ToolbarJsScope.Notifications)) {
    _notifications.lazyInit();
    _notificationsMode.lazyInit();

    if (_notifications.fetching || _notificationsMode.fetching) {
      fetching = true;
    } else {
      const notifs = _notifications.data as any[];
      data.count = notifs?.length || 0;
      data.dndActive = _notificationsMode.data !== NotificationsMode.All;
    }
  }

  if (scopes.includes(ToolbarJsScope.Media)) {
    _mediaSessions.lazyInit();
    _mediaDevices.lazyInit();

    if (_mediaSessions.fetching || _mediaDevices.fetching) {
      fetching = true;
    } else {
      const [mediaInputs, mediaOutputs] = _mediaDevices.data as any[];
      const defaultOutputDevice = (mediaOutputs as any[])?.find((d: any) => d.isDefaultMultimedia);
      const defaultInputDevice = (mediaInputs as any[])?.find((d: any) => d.isDefaultMultimedia);
      const defaultMediaSession = (_mediaSessions.data as any[])?.find((d: any) => d.default);
      const { volume = 0, muted: isMuted = true } = defaultOutputDevice || {};
      const { volume: inputVolume = 0, muted: inputIsMuted = true } = defaultInputDevice || {};
      data.defaultOutputDevice = defaultOutputDevice;
      data.defaultInputDevice = defaultInputDevice;
      data.volume = volume;
      data.isMuted = isMuted;
      data.inputVolume = inputVolume;
      data.inputIsMuted = inputIsMuted;
      data.mediaSession = defaultMediaSession || null;
    }
  }

  if (scopes.includes(ToolbarJsScope.Network)) {
    _networkOnline.lazyInit();
    _networkAdapters.lazyInit();
    _networkLocalIp.lazyInit();

    if (_networkOnline.fetching || _networkAdapters.fetching || _networkLocalIp.fetching) {
      fetching = true;
    } else {
      const interfaces = _networkAdapters.data as any[];
      const defaultIp = _networkLocalIp.data;
      const usingInterface = interfaces?.find((i: any) => i.ipv4 === defaultIp) || null;
      data.online = _networkOnline.data;
      data.interfaces = interfaces;
      data.usingInterface = usingInterface;
    }
  }

  if (scopes.includes(ToolbarJsScope.Keyboard)) {
    _languages.lazyInit();
    _imeState.lazyInit();

    if (_languages.fetching || _imeState.fetching) {
      fetching = true;
    } else {
      const languages = _languages.data as any[];
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
    }
  }

  if (scopes.includes(ToolbarJsScope.User)) {
    _user.lazyInit();

    if (_user.fetching) {
      fetching = true;
    } else {
      const user = _user.data as any;
      const allByWidget = settingsState.allByWidget;
      const userMenuConfig = allByWidget?.["@seelen/user-menu" as any] as
        | Record<string, unknown>
        | undefined;
      const source = userMenuConfig?.displayNameSource;
      const displayName = source === "xboxGamertag" && user?.xboxGamertag ? user.xboxGamertag : user?.name;
      data.user = { ...user, displayName };
    }
  }

  if (scopes.includes(ToolbarJsScope.Bluetooth)) {
    _bluetoothDevices.lazyInit();

    if (_bluetoothDevices.fetching) {
      fetching = true;
    } else {
      data.devices = _bluetoothDevices.data || [];
      data.getIconNameForBTDevice = getIconNameForBTDevice;
    }
  }

  if (scopes.includes(ToolbarJsScope.Power)) {
    _power.lazyInit();
    _powerMode.lazyInit();
    _batteries.lazyInit();

    if (_power.fetching || _powerMode.fetching || _batteries.fetching) {
      fetching = true;
    } else {
      data.power = _power.data;
      data.powerMode = _powerMode.data;
      data.batteries = _batteries.data;
    }
  }

  if (scopes.includes(ToolbarJsScope.FocusedApp)) {
    data.focusedApp = $state.snapshot(focused.value);
  }

  if (scopes.includes(ToolbarJsScope.Workspaces)) {
    const vd = $state.snapshot(virtualDesktop.value);
    const workspaces = vd?.workspaces || [];
    const activeWorkspace = vd?.active_workspace;
    data.workspaces = workspaces;
    data.activeWorkspace = activeWorkspace;
    data.onWheel = onWheelWorkspace;
  }

  if (scopes.includes(ToolbarJsScope.Disk)) {
    _disks.lazyInit();

    if (_disks.fetching) {
      fetching = true;
    } else {
      data.disks = _disks.data;
    }
  }

  if (scopes.includes(ToolbarJsScope.NetworkStatistics)) {
    _networkStatistics.lazyInit();

    if (_networkStatistics.fetching) {
      fetching = true;
    } else {
      data.networkStatistics = _networkStatistics.data;
    }
  }

  if (scopes.includes(ToolbarJsScope.Memory)) {
    _memory.lazyInit();

    if (_memory.fetching) {
      fetching = true;
    } else {
      data.memory = _memory.data;
    }
  }

  if (scopes.includes(ToolbarJsScope.Cpu)) {
    _cores.lazyInit();

    if (_cores.fetching) {
      fetching = true;
    } else {
      data.cores = _cores.data;
    }
  }

  return { fetching, data };
}

export { widgetStatuses };
