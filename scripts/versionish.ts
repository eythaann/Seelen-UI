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

function updateChangelog(version: string): void {
  if (version.includes("-")) {
    console.log("Skipping changelog update for pre-release version");
    return;
  }

  let content = fs.readFileSync("changelog.md", "utf-8");
  content = content.replace("# Changelog", `# Changelog\n\n## [${version}]`);
  fs.writeFileSync("changelog.md", content);
  console.log(`✓ Changelog updated for version ${version}`);
}

function createGitTag(version: string): void {
  const tag = `v${version}`;
  execSync(`git tag -s ${tag} -m "${tag}"`, { stdio: "inherit" });
  console.log(`✓ Git tag ${tag} created`);
}

async function main(args: string[]) {
  const argv = await yargs(hideBin(args))
    .version(false)
    .option("version", {
      type: "string",
      description: "Version to set (defaults to current package.json version)",
      demandOption: false,
    })
    .option("set-changelog", {
      type: "boolean",
      default: false,
      description: "Add a new changelog entry for the version",
    })
    .option("create-tag", {
      type: "boolean",
      default: false,
      description: "Create a signed git tag for the version",
    }).argv;

  // Get version from package.json if not provided
  const packageJson = JSON.parse(fs.readFileSync("./package.json", "utf-8"));
  const version = argv.version || packageJson.version;

  console.log(`Using version: ${version}`);

  const shouldSetChangelog = argv["set-changelog"];
  const shouldCreateTag = argv["create-tag"];

  // If only creating tag, skip file updates
  if (shouldCreateTag && !argv.version) {
    createGitTag(version);
    return;
  }

  // Update library versions
  console.log("Updating library versions...");
  updateCargoVersion("./libs/core/Cargo.toml", version);
  updateJsonVersion("./libs/core/deno.json", version);
  console.log("✓ Library versions updated");

  // Update app versions
  console.log("Updating app versions...");
  updateCargoVersion("./src/Cargo.toml", version);
  packageJson.version = version;
  fs.writeFileSync("./package.json", JSON.stringify(packageJson, null, 2) + "\n");
  console.log("✓ App versions updated");

  // Update changelog if requested
  if (shouldSetChangelog) {
    updateChangelog(version);
  }

  // Create tag if requested
  if (shouldCreateTag) {
    createGitTag(version);
  }

  console.log(`\n✓ Version ${version} set successfully`);

  // cargo check to update the lockfile
  execSync("cargo check", { stdio: "inherit" });
  // npm install to update the lockfile
  execSync("npm install", { stdio: "inherit" });
}

main(process.argv);
