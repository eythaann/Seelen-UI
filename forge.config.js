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
    extraResource: [path.join(process.cwd(), 'static/apps_templates')],
    asar: true,
    ignore: ['komorebi', 'scripts', '.vscode', '.gitignore', '.gitmodules', 'eslint.config.js', 'tsconfig.json', 'src'],
  },
  rebuildConfig: {},
  hooks: {
    generateAssets: async (forgeConfig, platform, version) => {
      fs.writeFileSync('./src/JsonSettings.interface.ts', await compileFromFile('./komorebi/schema.json'));

      fs.writeFileSync('./src/YamlSettings.interface.ts', await compileFromFile('./komorebi/schema.asc.json'));

      await import('./scripts/build.mjs');
    },
    prePackage: async (forgeConfig, platform, version) => {
      await runPwshScript('force_stop.ps1', `-Exeroute '${path.join(__dirname, './out/Komorebi UI-win32-x64/komorebi.exe')}'`);
    },
    packageAfterCopy: async (forgeConfig, buildPath, electronVersion, platform, arch) => {
      const ownLicense = fs.readFileSync(path.join(__dirname, 'LICENSE')).toString();
      const komorebiLicense = fs.readFileSync(path.join(__dirname, 'komorebi/LICENSE')).toString();
      const electronLicense = fs.readFileSync(path.join(buildPath, '../../LICENSE')).toString();

      fs.writeFileSync(
        path.join(buildPath, '../../LICENSE'),
        [ownLicense, komorebiLicense, electronLicense].join('\n\n- - -\n\n'),
      );

      // copy builded komorebi
      fs.copyFileSync(
        path.join(__dirname, 'komorebi/target/x86_64-pc-windows-msvc/release/komorebi.exe'),
        path.join(buildPath, 'komorebi.exe'),
      );
      fs.copyFileSync(
        path.join(__dirname, 'komorebi/target/x86_64-pc-windows-msvc/release/komorebic.exe'),
        path.join(buildPath, 'komorebic.exe'),
      );
    },
  },
  makers: [
    {
      name: '@electron-forge/maker-squirrel',
      config: {
        iconUrl: 'https://raw.githubusercontent.com/eythaann/Komorebi-UI/master/static/icons/icon.ico',
        setupIcon: path.join(process.cwd(), 'static/icons/icon.ico'),
      },
    },
    {
      name: '@electron-forge/maker-zip',
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
