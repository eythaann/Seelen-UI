import fs from "fs";
import process from "node:process";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import { execSync } from "child_process";

function updateCargoVersion(filePath: string, version: string): void {
  let content = fs.readFileSync(filePath, "utf-8");
  content = content.replace(/^version\s*=\s*".*"/m, `version = "${version}"`);
  fs.writeFileSync(filePath, content);
}

function updateJsonVersion(filePath: string, version: string): void {
  const json = JSON.parse(fs.readFileSync(filePath, "utf-8"));
  json.version = version;
  fs.writeFileSync(filePath, JSON.stringify(json, null, 2));
}

function updateChangelog(version: string, forceUpdate = false): void {
  if (version.includes("-") && !forceUpdate) {
    console.log("Skipping changelog update for pre-release version");
    return;
  }

  let content = fs.readFileSync("changelog.md", "utf-8");
  content = content.replace("# Changelog", `# Changelog\n\n## [${version}]`);
  fs.writeFileSync("changelog.md", content);
  console.log(`✓ Changelog updated for version ${version}`);
}

function replaceChangelogVersion(oldVersion: string, newVersion: string): void {
  let content = fs.readFileSync("changelog.md", "utf-8");
  content = content.replace(`## [${oldVersion}]`, `## [${newVersion}]`);
  fs.writeFileSync("changelog.md", content);
  console.log(`✓ Changelog version updated from ${oldVersion} to ${newVersion}`);
}

function createGitCommit(message: string): void {
  execSync("git add .", { stdio: "inherit" });
  execSync(`git commit -m "${message}"`, { stdio: "inherit" });
  console.log(`✓ Git commit created: ${message}`);
}

function createGitTag(tag: string): void {
  execSync(`git tag ${tag}`, { stdio: "inherit" });
  console.log(`✓ Git tag created: ${tag}`);
}

function updateAllVersions(version: string, skipLockfiles = false): void {
  // Update library versions
  console.log("Updating library versions...");
  updateCargoVersion("./libs/core/Cargo.toml", version);
  updateJsonVersion("./libs/core/deno.json", version);
  console.log("✓ Library versions updated");

  // Update app versions
  console.log("Updating app versions...");
  updateCargoVersion("./src/Cargo.toml", version);
  const packageJson = JSON.parse(fs.readFileSync("./package.json", "utf-8"));
  packageJson.version = version;
  fs.writeFileSync("./package.json", JSON.stringify(packageJson, null, 2) + "\n");
  console.log("✓ App versions updated");

  // Update lockfiles (skip in CI)
  if (!skipLockfiles) {
    execSync("cargo check", { stdio: "inherit" });
    execSync("npm install", { stdio: "inherit" });
  } else {
    console.log("⊘ Skipping lockfile updates (CI mode)");
  }
}

async function main(args: string[]) {
  await yargs(hideBin(args))
    .version(false)
    .command(
      "start <version>",
      "Start the development of a new release",
      (yargs) => {
        return yargs.positional("version", {
          type: "string",
          describe: "Version to start",
          demandOption: true,
        });
      },
      ({ version }) => {
        console.log(`Starting release with version: ${version}`);

        updateAllVersions(version);
        updateChangelog(`${version}-dev`, true);

        console.log(`\n✓ Version ${version} set successfully`);

        createGitCommit(`chore(release): start v${version}`);
      },
    )
    .command(
      "ci <version>",
      "Set version for CI builds (no git commit)",
      (yargs) => {
        return yargs.positional("version", {
          type: "string",
          describe: "Version to set",
          demandOption: true,
        });
      },
      ({ version }) => {
        console.log(`Setting version for CI: ${version}`);

        updateAllVersions(version, true);

        console.log(`\n✓ Version ${version} set successfully (no commit)`);
      },
    )
    .command(
      "finish",
      "Finish the current nightly release",
      () => {},
      () => {
        const packageJson = JSON.parse(fs.readFileSync("./package.json", "utf-8"));
        const currentVersion = packageJson.version;

        console.log(`Finishing release: ${currentVersion}`);

        replaceChangelogVersion(`${currentVersion}-dev`, currentVersion);

        console.log(`\n✓ Version ${currentVersion} set successfully`);

        createGitCommit(`chore(release): finish v${currentVersion}`);
        createGitTag(`v${currentVersion}`);
      },
    )
    .demandCommand(1, "You must provide a command (start, ci, or finish)")
    .help().argv;
}

main(process.argv);
