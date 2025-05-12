import { SystemLanguage } from '@seelen-ui/lib';
import {
  AppNotification,
  Battery,
  BluetoothDevice,
  DesktopWorkspace,
  FancyToolbarSettings,
  File,
  MediaDevice,
  MediaPlayer,
  NetworkAdapter,
  Placeholder,
  Plugin,
  PowerMode,
  PowerStatus,
  Settings,
  TrayIcon,
  User,
  WlanBssEntry,
  WorkspaceId,
} from '@seelen-ui/lib/types';

import { IRootState } from '../../../../../shared.interfaces';
import { FocusedApp } from '../../../../shared/interfaces/common';

export interface RootState extends IRootState<FancyToolbarSettings>, Pick<Settings, 'dateFormat'> {
  version: number;
  items: Placeholder;
  plugins: Plugin[];
  isOverlaped: boolean;
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
}
