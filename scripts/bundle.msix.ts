import { execSync } from "child_process";
import fs from "fs";
import path from "path";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";

import packageJson from "../package.json";
import process from "node:process";

async function getArgs() {
  const argv = await yargs(hideBin(process.argv)).option("target", {
    type: "string",
    description: "target to get the files from",
    default: "release",
  }).argv;
  return {
    target: argv.target,
  };
}

const [major, minor, patch, nightly_date] = packageJson.version.split(/[\.\+]/);
if (major === undefined || minor === undefined || patch === undefined) {
  throw new Error("Invalid package version");
}

const { target } = await getArgs();

console.info("Building MSIX...");
const buildFolder = `target/${target}/msix`;
const bundleFolder = `target/${target}/bundle/msix`;

fs.rmSync(buildFolder, { recursive: true, force: true }); // clean up
fs.mkdirSync(buildFolder, { recursive: true });
fs.mkdirSync(bundleFolder, { recursive: true });

// we skip revision here because greater numbers than 65535 are not supported on msix
const appxPackageVersion = `${major}.${minor}.${patch}.0`;
const fileVersion = nightly_date ? packageJson.version : appxPackageVersion;
const installer_msix_path = path.resolve(
  `${bundleFolder}/Seelen.SeelenUI_${fileVersion}_x64__p6yyn03m1894e.msix`,
);

// Add manifest
const manifest = fs
  .readFileSync("src/templates/AppxManifest.xml", "utf-8")
  .replace("{{version}}", appxPackageVersion);
fs.writeFileSync(`${buildFolder}/AppxManifest.xml`, manifest);

// Add binaries
fs.copyFileSync(`target/${target}/slu-service.exe`, `${buildFolder}/slu-service.exe`);
fs.copyFileSync(`target/${target}/seelen-ui.exe`, `${buildFolder}/seelen-ui.exe`);

// Add resources
fs.cpSync("src/static", `${buildFolder}/static`, { recursive: true });

try {
  // create installer bundle
  let out = execSync(`msixHeroCli pack -d ${buildFolder} -p ${installer_msix_path}`);
  console.info(out.toString());

  // sign installer with local certificate (this is for testing only) store changes the cert in the windows store
  let out2 = execSync(
    `msixHeroCli sign -f ./.cert/Seelen.pfx -p seelen -t http://time.certum.pl ${installer_msix_path}`,
  );
  console.info(out2.toString());
} catch (error) {
  console.error("\n\n", error?.toString());
  process.exit(1);
}
