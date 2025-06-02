import esbuild from 'esbuild';
import CssModulesPlugin from 'esbuild-css-modules-plugin';
import express from 'express';
import fs from 'fs';
import path from 'path';
import { renderToStaticMarkup } from 'react-dom/server';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';

async function getArgs() {
  const argv = await yargs(hideBin(process.argv))
    .option('production', {
      type: 'boolean',
      description: 'Enable Production Minified Bundle',
      default: false,
    })
    .option('serve', {
      type: 'boolean',
      description: 'Run a local server',
      default: false,
    }).argv;
  return {
    isProd: !!argv.production,
    serve: !!argv.serve,
  };
}

async function extractIconsIfNecessary() {
  if (fs.existsSync('./dist/icons')) {
    return;
  }

  console.info('Extracting SVG Lazy Icons');
  console.time('Lazy Icons');
  fs.mkdirSync('./dist/icons', { recursive: true });

  let tsFile = '// This file is generated on build, do not edit.\nexport type IconName =';
  const entries = fs.readdirSync('./node_modules/react-icons');

  for (const entry of entries) {
    const entryPath = path.join('./node_modules/react-icons', entry);
    const isDir = fs.statSync(entryPath).isDirectory();

    if (!isDir || entry === 'lib') {
      continue;
    }

    console.info('Extracting icon family:', entry);

    const family = await import(`react-icons/${entry}`);
    for (const [name, ElementConstructor] of Object.entries(family)) {
      if (typeof ElementConstructor !== 'function') {
        continue;
      }
      const element = ElementConstructor({ size: '1em' });
      const svg = renderToStaticMarkup(element);
      fs.writeFileSync(`./dist/icons/${name}.svg`, svg);
    }

    tsFile += `\n  | keyof typeof import('react-icons/${entry}')`;
  }

  tsFile += ';\n';
  fs.writeFileSync('./src/icons.ts', tsFile);
  console.timeEnd('Lazy Icons');
}

const appFolders = fs
  .readdirSync('src/apps')
  .filter((item) => item !== 'shared' && fs.statSync(path.join('src/apps', item)).isDirectory());

const entryPoints = appFolders
  .map((folder) => {
    const vanilla = `./src/apps/${folder}/index.ts`;
    const react = `./src/apps/${folder}/index.tsx`;
    const svelte = `./src/apps/${folder}/index.svelte`;
    if (fs.existsSync(vanilla)) {
      return vanilla;
    }
    if (fs.existsSync(react)) {
      return react;
    }
    if (fs.existsSync(svelte)) {
      return svelte;
    }
    return '';
  })
  .filter((file) => !!file);

entryPoints.push('./src/apps/shared/integrity.ts');

const OwnPlugin: esbuild.Plugin = {
  name: 'copy-public-by-entry',
  setup(build) {
    build.onStart(() => {
      console.time('build');
    });
    build.onEnd(() => {
      // copy public folder for each widget
      appFolders.forEach((folder) => {
        let source = `src/apps/${folder}/public`;
        let target = `dist/${folder}`;
        fs.cpSync(source, target, { recursive: true, force: true });
      });

      // move nested folders to root
      fs.readdirSync('dist/src/apps').forEach((folder) => {
        let source = `dist/src/apps/${folder}`;
        let target = `dist/${folder}`;
        fs.cpSync(source, target, { recursive: true, force: true });
      });
      fs.rmSync('dist/src', { recursive: true, force: true });

      console.timeEnd('build');
    });
  },
};

function startDevServer() {
  const app = express();
  app.use(express.static('dist'));
  app.listen(3579, () => {
    console.info('Listening on http://localhost:3579');
  });
}

(async function main() {
  const { isProd, serve } = await getArgs();

  await extractIconsIfNecessary();

  console.info('Removing old artifacts');
  // delete all in dist less icons
  fs.readdirSync('dist').forEach((folder) => {
    if (folder !== 'icons') {
      fs.rmSync(path.join('dist', folder), { recursive: true, force: true });
    }
  });

  const ctx = await esbuild.context({
    entryPoints: entryPoints,
    bundle: true,
    minify: isProd,
    sourcemap: !isProd,
    format: 'esm',
    outdir: './dist',
    jsx: 'automatic',
    loader: {
      '.yml': 'text',
    },
    plugins: [
      CssModulesPlugin({
        localsConvention: 'camelCase',
        pattern: 'do-not-use-on-themes-[local]-[hash]',
      }),
      OwnPlugin,
    ],
  });

  if (serve) {
    await ctx.watch();
    startDevServer();
  } else {
    await ctx.rebuild();
    await ctx.dispose();
  }
})();
