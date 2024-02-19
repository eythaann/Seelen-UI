import esbuild from 'esbuild';
import fs from 'fs';

const consolePrinter = {
  name: 'consolePrinter',
  setup(build) {
    build.onStart(() => {
      try {
        fs.mkdirSync('dist');
        fs.mkdirSync('dist/frontend-bundle');
      } catch (e) {}
      fs.cpSync('src/app/public', 'dist/frontend-bundle', {
        'recursive': true,
      });
    });

    build.onEnd(async (result) => {
      if (result.errors.length) {
        console.log(`\nFound ${result.errors.length} errors.`);
      }
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
  external: ['electron'],
  plugins: [
    consolePrinter,
  ]
});
