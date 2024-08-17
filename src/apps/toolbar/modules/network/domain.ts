export interface WlanBssEntry {
  ssid: string | null;
  bssid: string;
  channel_frequency: number;
  signal: number;
  connected: boolean;
  connected_channel: boolean;
}

export interface WlanProfile {
  profileName: string;
  ssid: string;
  authentication: string;
  encryption: string;
  password: string;
}
