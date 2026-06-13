import { execSync } from "child_process";
import fs from "fs";
import os from "os";
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

const outDir = target === "release" ? "target/release" : `target/${target}/release`;

console.info(`Building MSIX for ${arch}...`);
const buildFolder = `${outDir}/msix`;
const bundleFolder = `${outDir}/bundle/msix`;

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
fs.copyFileSync(`${outDir}/seelen-ui.exe`, `${buildFolder}/seelen-ui.exe`);
fs.copyFileSync(`${outDir}/slu-service.exe`, `${buildFolder}/slu-service.exe`);
fs.copyFileSync(`${outDir}/slu.exe`, `${buildFolder}/slu.exe`);

// add pdb files if debug
if (pre || target === "release") {
  fs.copyFileSync(`${outDir}/seelen_ui.pdb`, `${buildFolder}/seelen_ui.pdb`);
}

// dlls
fs.copyFileSync(`${outDir}/sluhk.dll`, `${buildFolder}/sluhk.dll`);

// integrity files
fs.copyFileSync(`${outDir}/SHA256SUMS`, `${buildFolder}/SHA256SUMS`);
fs.copyFileSync(`${outDir}/SHA256SUMS.sig`, `${buildFolder}/SHA256SUMS.sig`);

// Add resources
fs.cpSync("src/static", `${buildFolder}/static`, { recursive: true });

if (os.platform() !== "win32") {
  throw new Error("MSIX bundling is only supported on Windows");
}

const sdkBinRoot = "C:\\Program Files (x86)\\Windows Kits\\10\\bin";
const sdkVersion = fs
  .readdirSync(sdkBinRoot)
  .filter((d) => /^\d+\.\d+\.\d+\.\d+$/.test(d))
  .sort((a, b) => {
    const toNum = (v: string) =>
      v
        .split(".")
        .map(Number)
        .reduce((acc, n) => acc * 100000 + n, 0);
    return toNum(b) - toNum(a);
  })[0];

if (!sdkVersion) {
  throw new Error(
    "Windows SDK not found. Install it from https://developer.microsoft.com/windows/downloads/windows-sdk/",
  );
}

const sdkBin = path.join(sdkBinRoot, sdkVersion, "x64");
const makeappx = path.join(sdkBin, "makeappx.exe");
const signtool = path.join(sdkBin, "signtool.exe");
console.info(`Using Windows SDK ${sdkVersion} at ${sdkBin}`);

try {
  // create installer bundle
  let out = execSync(`"${makeappx}" pack /d "${buildFolder}" /p "${installer_msix_path}" /nv /o`);
  console.info(out.toString());

  // sign installer with local certificate (this is for testing only) store changes the cert in the windows store
  let out2 = execSync(
    `"${signtool}" sign /fd SHA256 /f ".cert/Seelen.pfx" /p Seelen /tr http://time.certum.pl /td sha256 "${installer_msix_path}"`,
  );
  console.info(out2.toString());
} catch (error) {
  console.error("\n\n", error?.toString());
  process.exit(1);
}
