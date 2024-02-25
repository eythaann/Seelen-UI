import { fromPackageRoot, getEnviroment, runPwshScript } from './utils';
import { app } from 'electron';
import { copyFileSync, existsSync, readFileSync, writeFileSync } from 'fs';
import path from 'path';

interface Version {
  version: string;
  path: string;
}

class Version {
  static path = getEnviroment() === 'installed' ? fromPackageRoot('../version') : fromPackageRoot('/version');

  static getVersionFile(): Version | undefined {
    if (!existsSync(Version.path)) {
      return;
    }
    return JSON.parse(readFileSync(Version.path, 'utf-8'));
  }

  static updateVersionFile(): Version {
    const data: Version = { version: app.getVersion(), path: fromPackageRoot() };
    writeFileSync(Version.path, JSON.stringify(data));
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

  const oldVersion = Version.getVersionFile();
  const currentVersion = Version.updateVersionFile();

  // execute install and update tasks
  if (!oldVersion || oldVersion.version != currentVersion.version) {
    extract('komorebi.exe');
    extract('komorebic.exe');
    await runPwshScript('update_path.ps1', `-AppPath "${currentVersion.path}\\"`);
    if (oldVersion) {
      await runPwshScript('update_path.ps1', `-AppPath "${oldVersion.path}\\" -Delete $true`);
      await runPwshScript('force_stop.ps1', `-ExeRoute "${fromPackageRoot('komorebi.exe')}"`);
    }
  }

  await runPwshScript('manual_run.ps1', `-ExeRoute "${fromPackageRoot('komorebi.exe')}"`);
};
