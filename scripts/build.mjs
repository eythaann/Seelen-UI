import esbuild from 'esbuild';
import fs from 'fs';

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
  minify: true,
  sourcemap: true,
  outfile: './dist/frontend-bundle/bundle.js',
  jsx: 'automatic',
  plugins: [CopyPublic],
});

await esbuild.build({
  entryPoints: ['./src/background/index.ts', './src/background/preload.ts'],
  bundle: true,
  minify: false,
  outdir: './dist/background-bundle',
  platform: 'node',
  external: ['electron'],
});
