import esbuild from 'esbuild';
import fs from 'fs';
import path from 'path';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';

function cleanOldFiles() {
  if (fs.existsSync('dist')) {
    fs.rmSync('dist', { recursive: true });
  }
}

async function main() {
  console.time('Build');
  cleanOldFiles();

  const argv = await yargs(hideBin(process.argv)).option('production', {
    type: 'boolean',
    description: 'Enable Production Minified Bundle',
  }).argv;

  const isProdMode = !!argv.production;

  const appFolders = fs
    .readdirSync('src/apps')
    .filter((item) => item !== 'shared' && fs.statSync(path.join('src/apps', item)).isDirectory());

  await esbuild.build({
    entryPoints: appFolders.map((folder) => `./src/apps/${folder}/index.tsx`),
    bundle: true,
    minify: isProdMode,
    sourcemap: !isProdMode,
    outdir: './dist',
    jsx: 'automatic',
    define: {
      'process.env': JSON.stringify({
        packageVersion: JSON.parse(fs.readFileSync('package.json', 'utf-8')).version,
      }),
    },
    loader: {
      '.yml': 'text',
    },
  });

  appFolders.forEach((folder) => {
    fs.cpSync(`src/apps/${folder}/index.html`, `dist/${folder}/index.html`);
  });

  console.timeEnd('Build');
}

main();
