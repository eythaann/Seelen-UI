import esbuild from 'esbuild';
import fs from 'fs';
import path from 'path';
import { renderToStaticMarkup } from 'react-dom/server';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';

async function extractIconsIfNecessary() {
  if (fs.existsSync('./dist/icons')) {
    return;
  }

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
    for (const [name, ElementConstructor] of Object.entries(family.default)) {
      const element = ElementConstructor({ size: '1em' });
      const svg = renderToStaticMarkup(element);
      fs.writeFileSync(`./dist/icons/${name}.svg`, svg);
    }
  }

  console.timeEnd('Bundle Lazy Icons');
}

async function main() {
  console.time('Build UI');
  const argv = await yargs(hideBin(process.argv)).option('production', {
    type: 'boolean',
    description: 'Enable Production Minified Bundle',
  }).argv;

  const isProdMode = !!argv.production;

  const appFolders = fs
    .readdirSync('src/apps')
    .filter((item) => item !== 'shared' && fs.statSync(path.join('src/apps', item)).isDirectory());

  // remove previous build
  appFolders.forEach((folder) => {
    const filePath = path.join('dist', folder);
    if (fs.existsSync(filePath)) {
      fs.rmSync(filePath, { recursive: true, force: true });
    }
  });

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

  await esbuild.build({
    entryPoints: entryPoints,
    bundle: true,
    minify: isProdMode,
    sourcemap: !isProdMode,
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
  });

  appFolders.forEach((folder) => {
    let source = `src/apps/${folder}/public`;
    let target = `dist/${folder}`;
    fs.cpSync(source, target, { recursive: true });
  });
  console.timeEnd('Build UI');

  await extractIconsIfNecessary();
}

main();
