import type { SystemLanguage } from "@seelen-ui/lib";
import type {
  AppNotification,
  Battery,
  BluetoothDevice,
  File,
  MediaDevice,
  MediaPlayer,
  NetworkAdapter,
  PowerMode,
  PowerStatus,
  User,
  WlanBssEntry,
} from "@seelen-ui/lib/types";

export interface RootState {
  version: number;
  user: User | null;
  userRecentFolder: File[];
  userDesktopFolder: File[];
  userDocumentsFolder: File[];
  userDownloadsFolder: File[];
  userPicturesFolder: File[];
  userVideosFolder: File[];
  userMusicFolder: File[];
  env: Record<string, string>;
  bluetoothDevices: BluetoothDevice[];
  discoveredBluetoothDevices: BluetoothDevice[];
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
