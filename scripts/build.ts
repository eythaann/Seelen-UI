import esbuild from 'esbuild';
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

  console.info('Extracting SVG Icons');
  console.time('Bundle Lazy Icons');
  fs.mkdirSync('./dist/icons', { recursive: true });

  const promises = [
    import('react-icons/ai'),
    import('react-icons/bi'),
    import('react-icons/bs'),
    import('react-icons/cg'),
    import('react-icons/ci'),
    import('react-icons/di'),
    import('react-icons/fa'),
    import('react-icons/fa6'),
    import('react-icons/fc'),
    import('react-icons/fi'),
    import('react-icons/gi'),
    import('react-icons/go'),
    import('react-icons/gr'),
    import('react-icons/hi'),
    import('react-icons/hi2'),
    import('react-icons/im'),
    import('react-icons/io'),
    import('react-icons/io5'),
    import('react-icons/lia'),
    import('react-icons/lu'),
    import('react-icons/md'),
    import('react-icons/pi'),
    import('react-icons/ri'),
    import('react-icons/rx'),
    import('react-icons/si'),
    import('react-icons/sl'),
    import('react-icons/tb'),
    import('react-icons/tfi'),
    import('react-icons/ti'),
    import('react-icons/vsc'),
    import('react-icons/wi'),
  ];

  let families = await Promise.all(promises);
  for (const family of families) {
    for (const [name, ElementConstructor] of Object.entries(family)) {
      const element = ElementConstructor({ size: '1em' });
      const svg = renderToStaticMarkup(element);
      fs.writeFileSync(`./dist/icons/${name}.svg`, svg);
    }
  }

  console.timeEnd('Bundle Lazy Icons');
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

const copyPublicByEntry: esbuild.Plugin = {
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

void (async function main() {
  const { isProd, serve } = await getArgs();

  await extractIconsIfNecessary();

  console.info('Removing old artifacts');
  appFolders.forEach((folder) => {
    const filePath = path.join('dist', folder);
    if (fs.existsSync(filePath)) {
      fs.rmSync(filePath, { recursive: true, force: true });
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
    define: {
      'process.env': JSON.stringify({
        packageVersion: JSON.parse(fs.readFileSync('package.json', 'utf-8')).version,
      }),
    },
    loader: {
      '.yml': 'text',
    },
    plugins: [copyPublicByEntry],
  });

  if (serve) {
    await ctx.watch();
    startDevServer();
  } else {
    await ctx.rebuild();
    await ctx.dispose();
  }
})();
