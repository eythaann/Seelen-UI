import { config as loadEnv } from 'dotenv';
import esbuild from 'esbuild';
import fs from 'fs';
import { compileFromFile } from 'json-schema-to-typescript';
import path from 'path';
import toml from 'toml';

const { GITHUB_TOKEN: _, ...parsedEnv } = loadEnv().parsed!;

async function main() {
  fs.writeFileSync('src/JsonSettings.interface.ts', await compileFromFile('komorebi/schema.json'));

  fs.writeFileSync(
    'src/YamlSettings.interface.ts',
    await compileFromFile('komorebi/schema.asc.json'),
  );

  const appFolders = fs
    .readdirSync('src/apps')
    .filter((item) => item !== 'utils' && fs.statSync(path.join('src/apps', item)).isDirectory());

  await esbuild.build({
    entryPoints: appFolders.map((folder) => `./src/apps/${folder}/index.tsx`),
    bundle: true,
    minify: false,
    sourcemap: true,
    outdir: './dist',
    jsx: 'automatic',
    define: {
      'process.env': JSON.stringify({
        ...(parsedEnv || {}),
        packageVersion: JSON.parse(fs.readFileSync('package.json', 'utf-8')).version,
        komorebiVersion: toml.parse(fs.readFileSync('komorebi/komorebi/Cargo.toml', 'utf-8'))
          .package.version,
      }),
    },
  });

  appFolders.forEach((folder) => {
    fs.cpSync(`src/apps/${folder}/index.html`, `dist/${folder}/index.html`);
  });
}

main();
