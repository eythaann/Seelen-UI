import type {
  AppNotification,
  Battery,
  BluetoothDevice,
  MediaDevice,
  MediaPlayer,
  NetworkAdapter,
  PowerMode,
  PowerStatus,
  SystemLanguage,
  User,
  WlanBssEntry,
} from "@seelen-ui/lib/types";

export interface RootState {
  version: number;
  user: User | null;
  env: Record<string, string>;
  bluetoothDevices: BluetoothDevice[];
  powerStatus: PowerStatus;
  powerPlan: PowerMode;
  batteries: Battery[];
  networkAdapters: NetworkAdapter[];
  networkLocalIp: string | null;
  online: boolean;
  wlanBssEntries: WlanBssEntry[];
  mediaSessions: MediaPlayer[];
  mediaOutputs: MediaDevice[];
  mediaInputs: MediaDevice[];
  notifications: AppNotification[];
  languages: SystemLanguage[];
}
