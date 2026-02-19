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

const [major, minor, patch, pre, _build_number] = packageJson.version.split(/[\.\+\-]/);
if (major === undefined || minor === undefined || patch === undefined) {
  throw new Error("Invalid package version");
}

const { target } = await getArgs();

// Determine architecture for MSIX naming
const archMap: Record<string, string> = {
  "x86_64-pc-windows-msvc": "x64",
  "aarch64-pc-windows-msvc": "arm64",
};
const arch = archMap[target] || "x64";

console.info(`Building MSIX for ${arch}...`);
const buildFolder = `target/${target}/release/msix`;
const bundleFolder = `target/${target}/release/bundle/msix`;

fs.rmSync(buildFolder, { recursive: true, force: true }); // clean up
fs.mkdirSync(buildFolder, { recursive: true });
fs.mkdirSync(bundleFolder, { recursive: true });

// we skip revision here because greater numbers than 65535 are not supported on msix
const appxPackageVersion = `${major}.${minor}.${patch}.0`;
const fileVersion = pre ? packageJson.version : appxPackageVersion;
const installer_msix_path = path.resolve(`${bundleFolder}/Seelen.UI_${fileVersion}_${arch}.msix`);

// Add manifest
const manifest = fs
  .readFileSync("src/templates/AppxManifest.xml", "utf-8")
  .replace("{{version}}", appxPackageVersion)
  .replace("{{architecture}}", arch);
fs.writeFileSync(`${buildFolder}/AppxManifest.xml`, manifest);

// Add binaries
fs.copyFileSync(`target/${target}/release/seelen-ui.exe`, `${buildFolder}/seelen-ui.exe`);
fs.copyFileSync(`target/${target}/release/slu-service.exe`, `${buildFolder}/slu-service.exe`);

// add pdb files if debug
if (pre || target === "./") {
  fs.copyFileSync(`target/${target}/release/seelen_ui.pdb`, `${buildFolder}/seelen_ui.pdb`);
}

// dlls
fs.copyFileSync(`target/${target}/release/sluhk.dll`, `${buildFolder}/sluhk.dll`);

// integrity files
fs.copyFileSync(`target/${target}/release/SHA256SUMS`, `${buildFolder}/SHA256SUMS`);
fs.copyFileSync(`target/${target}/release/SHA256SUMS.sig`, `${buildFolder}/SHA256SUMS.sig`);

// Add resources
fs.cpSync("src/static", `${buildFolder}/static`, { recursive: true });

try {
  // create installer bundle
  let out = execSync(`msixHeroCli pack -d ${buildFolder} -p ${installer_msix_path}`);
  console.info(out.toString());

  // sign installer with local certificate (this is for testing only) store changes the cert in the windows store
  let out2 = execSync(
    `msixHeroCli sign -f ./.cert/Seelen.pfx -p Seelen -t http://time.certum.pl ${installer_msix_path}`,
  );
  console.info(out2.toString());
} catch (error) {
  console.error("\n\n", error?.toString());
  process.exit(1);
}
