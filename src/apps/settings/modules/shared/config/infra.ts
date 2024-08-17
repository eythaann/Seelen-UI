import { path } from '@tauri-apps/api';

export const EnvConfig = {
  version: process.env.packageVersion,
};

export async function resolveDataPath(...sub: string[]) {
  return await path.join(await path.appDataDir(), ...sub);
}