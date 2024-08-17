import packageJson from '../package.json';
import tauriConfig from '../tauri.conf.json';
import { execSync } from 'child_process';
import fs from 'fs';
import glob from 'glob';
import path from 'path';

console.info('Building MSIX...');
const msixCmdsPath = path.resolve('target/release/msix/commands.txt');
const msixTemplatePath = path.resolve('templates/installer.msix');

const packageVersion = packageJson.version + '.0';
const installer_msix_path = path.resolve(
  `target/release/bundle/msix/Seelen.SeelenUI_${packageVersion}_x64__p6yyn03m1894e.msix`,
);

if (fs.existsSync(msixCmdsPath)) {
  fs.rmSync(msixCmdsPath);
} else {
  fs.mkdirSync(path.dirname(msixCmdsPath));
}

fs.appendFileSync(msixCmdsPath, `setIdentity --packageVersion ${packageVersion}\n`);

fs.appendFileSync(
  msixCmdsPath,
  `addFile --target "${packageJson.productName}.exe" --source "${path.resolve(
    `target/release/${packageJson.productName}.exe`,
  )}"\n`,
);

tauriConfig.bundle.resources.forEach((pattern) => {
  let files = glob.sync(pattern, { nodir: true });
  files.forEach((file) => {
    fs.appendFileSync(
      msixCmdsPath,
      `addFile --target "${file}" --source "${path.resolve(`target/release/${file}`)}"\n`,
    );
  });
});

try {
  fs.mkdirSync(installer_msix_path.split(path.sep).slice(0, -1).join(path.sep), {
    recursive: true,
  });
  fs.copyFileSync(msixTemplatePath, installer_msix_path);

  const buffer = execSync(`msixHeroCli edit "${installer_msix_path}" list "${msixCmdsPath}"`);
  console.info(buffer.toString());
} catch (error) {
  console.error('\n', error);
}
