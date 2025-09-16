import { app, path } from "@tauri-apps/api";

export const EnvConfig = {
  version: await app.getVersion(),
};

export async function resolveDataPath(...sub: string[]) {
  return await path.join(await path.appDataDir(), ...sub);
}
