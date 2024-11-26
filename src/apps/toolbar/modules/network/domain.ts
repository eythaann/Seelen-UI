export interface WlanBssEntry {
  ssid: string | null;
  bssid: string;
  channelFrequency: number;
  signal: number;
  connected: boolean;
  connectedChannel: boolean;
  secured: boolean;
  known: boolean;
}

export interface WlanProfile {
  profileName: string;
  ssid: string;
  authentication: string;
  encryption: string;
  password: string;
}
