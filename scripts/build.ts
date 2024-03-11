import { config as loadEnv } from 'dotenv';
import esbuild from 'esbuild';
import fs from 'fs';
import { compileFromFile } from 'json-schema-to-typescript';
import toml from 'toml';

const { GITHUB_TOKEN: _, ...parsedEnv } = loadEnv().parsed!;

async function main() {
  fs.writeFileSync('src/JsonSettings.interface.ts', await compileFromFile('komorebi/schema.json'));

  fs.writeFileSync(
    'src/YamlSettings.interface.ts',
    await compileFromFile('komorebi/schema.asc.json'),
  );

  await esbuild.build({
    entryPoints: ['./src/apps/settings/index.tsx', './src/apps/seelenpad/index.tsx'],
    bundle: true,
    minify: true,
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

  fs.cpSync('src/apps/settings/index.html', 'dist/settings/index.html');
  fs.cpSync('src/apps/seelenpad/index.html', 'dist/seelenpad/index.html');
}

main();
