import { fromPackageRoot, runPwshScript } from './utils';
import { app } from 'electron';
import { copyFileSync, existsSync, readFileSync, writeFileSync } from 'fs';
import { ensureFileSync } from 'fs-extra';
import path from 'path';

export interface Version {
  version: string;
  path: string;
}

export class Version {
  static pathToSave = path.join(app.getPath('userData'), 'version');

  static getVersionFile(): Version | undefined {
    if (!existsSync(Version.pathToSave)) {
      return;
    }
    return JSON.parse(readFileSync(Version.pathToSave, 'utf-8'));
  }

  static updateVersionFile(): Version {
    const data: Version = { version: app.getVersion(), path: fromPackageRoot() };
    ensureFileSync(Version.pathToSave);
    writeFileSync(Version.pathToSave, JSON.stringify(data));
    return data;
  }
}

const extract = (file: string) => {
  let destinationPath = fromPackageRoot(file);
  if (!existsSync(destinationPath)) {
    copyFileSync(path.join(app.getAppPath(), file), destinationPath);
  }
};

export const init = async () => {
  if (!app.isPackaged) {
    return;
  }

  extract('komorebi.exe');
  extract('komorebic.exe');

  const oldVersion = Version.getVersionFile();
  const currentVersion = Version.updateVersionFile();

  // fresh install or update
  if (!oldVersion || oldVersion.version != currentVersion.version) {
    await runPwshScript('update_path.ps1', `-AppPath "${currentVersion.path}\\"`);

    if (oldVersion) {
      await runPwshScript('update_path.ps1', `-AppPath "${oldVersion.path}\\" -Delete true`);
      await runPwshScript('force_stop.ps1', `-ExeRoute "${fromPackageRoot('komorebi.exe')}"`);
    }
  }

  await runPwshScript('manual_run.ps1', `-ExeRoute "${fromPackageRoot('komorebi.exe')}"`);
};
