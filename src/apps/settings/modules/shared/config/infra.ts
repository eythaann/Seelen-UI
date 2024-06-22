import { path } from '@tauri-apps/api';

export const EnvConfig = {
  version: process.env.packageVersion,
};

export async function resolveDotConfigPath(...sub: string[]) {
  return await path.join(await path.homeDir(), '.config', 'seelen', ...sub);
}