import { config as loadEnv } from 'dotenv';
import esbuild from 'esbuild';
import fs, { readFileSync } from 'fs';
import toml from 'toml';

const { GITHUB_TOKEN: _, ...parsedEnv } = loadEnv();

const CopyPublic = {
  name: 'CopyPublic',
  setup(build) {
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

await esbuild.build({
  entryPoints: ['./src/app/index.tsx'],
  bundle: true,
  minify: false,
  outfile: './dist/frontend-bundle/bundle.js',
  jsx: 'automatic',
  plugins: [CopyPublic],
  define: {
    'process.env': JSON.stringify({
      ...(parsedEnv || {}),
      packageVersion: JSON.parse(readFileSync('package.json', 'utf-8')).version,
      komorebiVersion: toml.parse(readFileSync('komorebi/komorebi/Cargo.toml', 'utf-8')).package.version,
    }),
  },
});

await esbuild.build({
  entryPoints: ['./src/background/index.ts', './src/background/preload.ts'],
  bundle: true,
  minify: false,
  outdir: './dist/background-bundle',
  platform: 'node',
  external: ['electron'],
});
