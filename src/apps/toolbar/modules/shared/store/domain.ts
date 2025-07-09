import { SystemLanguage } from '@seelen-ui/lib';
import {
  AppNotification,
  Battery,
  BluetoothDevice,
  DesktopWorkspace,
  File,
  FocusedApp,
  MediaDevice,
  MediaPlayer,
  NetworkAdapter,
  PowerMode,
  PowerStatus,
  TrayIcon,
  User,
  WegAppGroupItem,
  WlanBssEntry,
  WorkspaceId,
} from '@seelen-ui/lib/types';

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
  focused: FocusedApp | null;
  env: Record<string, string>;
  bluetoothDevices: BluetoothDevice[];
  discoveredBluetoothDevices: BluetoothDevice[];
  powerStatus: PowerStatus;
  powerPlan: PowerMode;
  batteries: Battery[];
  workspaces: DesktopWorkspace[];
  activeWorkspace: WorkspaceId | null;
  systemTray: TrayIcon[];
  networkAdapters: NetworkAdapter[];
  networkLocalIp: string | null;
  online: boolean;
  wlanBssEntries: WlanBssEntry[];
  mediaSessions: MediaPlayer[];
  mediaOutputs: MediaDevice[];
  mediaInputs: MediaDevice[];
  notifications: AppNotification[];
  languages: SystemLanguage[];
  openApps: WegAppGroupItem[];
  windowColorByHandle: Record<string, { background: string; foreground: string }>;
}
