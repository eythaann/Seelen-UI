import { invoke } from '@tauri-apps/api/core';
import { MonitorInfo } from 'seelen-core';

export const loadMonitorInfo = async (): Promise<MonitorInfo> => {
  return invoke<MonitorInfo>('get_monitor_info');
};