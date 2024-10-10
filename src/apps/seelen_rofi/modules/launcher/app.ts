import { path } from '@tauri-apps/api';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import yaml from 'js-yaml';
import { LauncherHistory } from 'seelen-core';

export async function SaveHistory(history: LauncherHistory) {
  const yaml_route = await path.join(await path.appDataDir(), 'history');
  await writeTextFile(yaml_route, yaml.dump(history));
}
