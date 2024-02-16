import esbuild from 'esbuild';
import fs from 'fs';

const consolePrinter = {
  name: 'consolePrinter',
  setup(build) {
    build.onStart(() => {
      try {
        fs.mkdirSync('dist');
      } catch (e) {}
      try {
        fs.cpSync('src/app/public', 'dist', {
          'recursive': true,
        });
      } catch (e) {}
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
  outfile: './dist/bundle.js',
  jsx: 'automatic',
  plugins: [
    consolePrinter,
  ]
});
