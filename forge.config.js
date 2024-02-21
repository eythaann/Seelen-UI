/* eslint-disable @ts/no-unused-vars */
const path = require('path');
const fs = require('fs');
const { runPwshScript } = require('./src/background/utils');
const { compileFromFile } = require('json-schema-to-typescript');

/**
 * @typedef {import('@electron-forge/shared-types').ForgeConfig} ForgeConfig
 * @type {ForgeConfig}
 */
const config = {
  packagerConfig: {
    name: 'Komorebi UI',
    executableName: 'komorebi-ui',
    icon: path.join(process.cwd(), 'static/icons/icon'),
    asar: true,
    ignore: ['komorebi', 'scripts', '.vscode', '.gitignore', '.gitmodules', 'eslint.config.js', 'tsconfig.json', 'src'],
  },
  rebuildConfig: {},
  hooks: {
    generateAssets: async (forgeConfig, platform, version) => {
      fs.writeFileSync(
        './src/JsonSettings.interface.ts',
        await compileFromFile('./komorebi/schema.json'),
      );

      fs.writeFileSync(
        './src/YamlSettings.interface.ts',
        await compileFromFile('./komorebi/schema.asc.json'),
      );

      await import('./scripts/build.mjs');
    },
    prePackage: async (forgeConfig, platform, version) => {
      await runPwshScript('force_stop.ps1');
    },
    packageAfterExtract: async (forgeConfig, buildPath, electronVersion, platform, arch) => {
      const licensesPath = path.join(buildPath, 'licenses');

      if (!fs.existsSync(licensesPath)) {
        fs.mkdirSync(licensesPath);
      }

      fs.renameSync(path.join(buildPath, 'version'), path.join(licensesPath, 'version.electron'));
      fs.renameSync(path.join(buildPath, 'LICENSE'), path.join(licensesPath, 'LICENSE.electron'));
      fs.renameSync(
        path.join(buildPath, 'LICENSES.chromium.html'),
        path.join(licensesPath, 'LICENSES.electron.chromium.html'),
      );
    },
    packageAfterCopy: async (forgeConfig, buildPath, electronVersion, platform, arch) => {
      const licensesPath = path.join(buildPath, '../../licenses');
      fs.copyFileSync(path.join(__dirname, 'LICENSE'), path.join(licensesPath, 'LICENSE'));
      fs.copyFileSync(path.join(__dirname, 'komorebi/LICENSE'), path.join(licensesPath, 'LICENSE.komorebi'));

      // copy builded komorebi
      fs.copyFileSync(
        path.join(__dirname, 'komorebi/target/x86_64-pc-windows-msvc/release/komorebi.exe'),
        path.join(buildPath, '../../komorebi.exe'),
      );
      fs.copyFileSync(
        path.join(__dirname, 'komorebi/target/x86_64-pc-windows-msvc/release/komorebic.exe'),
        path.join(buildPath, '../../komorebic.exe'),
      );
    },
  },
  makers: [
    {
      name: '@electron-forge/maker-squirrel',
      config: {
        setupIcon: path.join(process.cwd(), 'static/icons/icon.ico'),
      },
    },
  ],
  plugins: [
    {
      name: '@electron-forge/plugin-auto-unpack-natives',
      config: {},
    },
  ],
};

module.exports = config;
