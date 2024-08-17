import esbuild from 'esbuild';
import fs from 'fs';
import path from 'path';

async function main() {
  const appFolders = fs
    .readdirSync('src/apps')
    .filter((item) => item !== 'shared' && fs.statSync(path.join('src/apps', item)).isDirectory());

  await esbuild.build({
    entryPoints: appFolders.map((folder) => `./src/apps/${folder}/index.tsx`),
    bundle: true,
    minify: false,
    sourcemap: true,
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
}

main();
