import { execSync } from 'child_process';
import fs from 'fs';
import glob from 'glob';
import path from 'path';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';

import packageJson from '../package.json';
import tauriConfig from '../tauri.conf.json';

async function getArgs() {
  const argv = await yargs(hideBin(process.argv)).option('target', {
    type: 'string',
    description: 'target to get the files from',
    default: 'release',
  }).argv;
  return {
    target: argv.target,
  };
}

void async function main() {
  const { target } = await getArgs();

  console.info('Building MSIX...');
  const msixCmdsPath = path.resolve(`target/${target}/msix/commands.txt`);
  const msixTemplatePath = path.resolve('templates/installer.msix');

  const [major, minor, patch, nightly_date = 0] = packageJson.version.split(/[\.\+]/);
  if (major === undefined || minor === undefined || patch === undefined) {
    throw new Error('Invalid package version');
  }

  // we skip revision here because greater numbers are not supported on msix
  const packageVersion = `${major}.${minor}.${patch}.0`;
  const postfix = nightly_date ? `+${nightly_date}` : '';
  const installer_msix_path = path.resolve(
    `target/${target}/bundle/msix/Seelen.SeelenUI_${packageVersion}${postfix}_x64__p6yyn03m1894e.msix`,
  );

  if (!fs.existsSync(msixCmdsPath)) {
    fs.mkdirSync(path.dirname(msixCmdsPath), { recursive: true });
  }
  fs.writeFileSync(msixCmdsPath, '');

  // Set app version
  fs.appendFileSync(msixCmdsPath, `setIdentity --packageVersion ${packageVersion}\n`);

  // Add main binary
  fs.appendFileSync(
    msixCmdsPath,
    `addFile --target "${packageJson.productName}.exe" --source "${path.resolve(
      `target/${target}/${packageJson.productName}.exe`,
    )}"\n`,
  );

  // Add crash service
  fs.appendFileSync(
    msixCmdsPath,
    `addFile --target "slu-service.exe" --source "${path.resolve(
      `target/${target}/slu-service.exe`,
    )}"\n`,
  );

  // Add resources
  tauriConfig.bundle.resources.forEach((pattern) => {
    let files = glob.sync(pattern, { nodir: true });
    files.forEach((file) => {
      fs.appendFileSync(
        msixCmdsPath,
        `addFile --target "${file}" --source "${path.resolve(`target/${target}/${file}`)}"\n`,
      );
    });
  });

  try {
    fs.mkdirSync(installer_msix_path.split(path.sep).slice(0, -1).join(path.sep), {
      recursive: true,
    });
    fs.copyFileSync(msixTemplatePath, installer_msix_path);

    /* let buffer = execSync(
      'msixherocli.exe newcert --directory ./.cert --name Seelen --password seelen --subject CN=7E60225C-94CB-4B2E-B17F-0159A11074CB',
    );
    console.info(buffer.toString()); */

    let buffer = execSync(`msixHeroCli edit "${installer_msix_path}" list "${msixCmdsPath}"`);
    console.info(buffer.toString());

    buffer = execSync(
      `msixHeroCli sign -f ./.cert/Seelen.pfx -p seelen -t http://time.certum.pl ${installer_msix_path}`,
    );
    console.info(buffer.toString());
  } catch (error) {
    console.error('\n\n', error);
    process.exit(1);
  }
}();

