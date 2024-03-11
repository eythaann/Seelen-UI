import { config as loadEnv } from 'dotenv';
import esbuild, { PluginBuild } from 'esbuild';
import fs from 'fs';
import { compileFromFile } from 'json-schema-to-typescript';
import toml from 'toml';

const { GITHUB_TOKEN: _, ...parsedEnv } = loadEnv().parsed!;

const CopyPublic = {
  name: 'CopyPublic',
  setup(build: PluginBuild) {
    build.onStart(() => {
      try {
        fs.mkdirSync('dist');
        fs.mkdirSync('dist/frontend-bundle');
      } catch (e) {}
      fs.cpSync('src/app/public', 'dist/frontend-bundle', {
        recursive: true,
      });
    });
  },
};

async function main() {
  fs.writeFileSync(
    'src/JsonSettings.interface.ts',
    await compileFromFile('komorebi/schema.json'),
  );

  fs.writeFileSync(
    'src/YamlSettings.interface.ts',
    await compileFromFile('komorebi/schema.asc.json'),
  );

  await esbuild.build({
    entryPoints: ['./src/app/index.tsx', './src/apps/seelenpad/index.tsx'],
    bundle: true,
    minify: true,
    sourcemap: true,
    outdir: './dist',
    jsx: 'automatic',
    plugins: [],
    define: {
      'process.env': JSON.stringify({
        ...(parsedEnv || {}),
        packageVersion: JSON.parse(fs.readFileSync('package.json', 'utf-8')).version,
        komorebiVersion: toml.parse(fs.readFileSync('komorebi/komorebi/Cargo.toml', 'utf-8')).package.version,
      }),
    },
  });
}

main();