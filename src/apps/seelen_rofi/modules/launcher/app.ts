import { path } from '@tauri-apps/api';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import yaml from 'js-yaml';

export async function SaveHistory(history: Record<string, string[]>) {
  const yaml_route = await path.join(await path.appDataDir(), 'history');
  await writeTextFile(yaml_route, yaml.dump(history));
}
