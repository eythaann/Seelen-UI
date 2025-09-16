import fs from "fs";
import process from "node:process";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import { execSync } from "child_process";

async function main(args: string[]) {
  const argv = await yargs(hideBin(args))
    .version(false)
    .option("version", {
      type: "string",
      description: "Version to set",
      demandOption: true,
    })
    .option("start", {
      type: "boolean",
      default: false,
      description: "Starting the development of a new version",
    })
    .option("finish", {
      type: "boolean",
      default: false,
      description: "Finishing the development of a new version",
    }).argv;

  // update library version
  {
    let content = fs.readFileSync("./libs/core/Cargo.toml", "utf-8");
    content = content.replace(
      /^version\s*=\s*".*"/m,
      `version = "${argv.version}"`,
    );
    fs.writeFileSync("./libs/core/Cargo.toml", content);

    let denoJson = JSON.parse(
      fs.readFileSync("./libs/core/deno.json", "utf-8"),
    );
    denoJson.version = argv.version;
    fs.writeFileSync(
      "./libs/core/deno.json",
      JSON.stringify(denoJson, null, 2),
    );
  }

  // update app version
  {
    let content = fs.readFileSync("./src/Cargo.toml", "utf-8");
    content = content.replace(
      /^version\s*=\s*".*"/m,
      `version = "${argv.version}"`,
    );
    fs.writeFileSync("./src/Cargo.toml", content);

    let packageJson = JSON.parse(fs.readFileSync("./package.json", "utf-8"));
    packageJson.version = argv.version;
    fs.writeFileSync("./package.json", JSON.stringify(packageJson, null, 2));
  }

  if (argv.start) {
    // update changelog only on release channel
    if (!argv.version.includes("-")) {
      let content = fs.readFileSync("changelog.md", "utf-8");
      content = content.replace(
        "# Changelog",
        `# Changelog\n\n## [${argv.version}]`,
      );
      fs.writeFileSync("changelog.md", content);
    }
  }

  if (argv.finish) {
    execSync(`git tag -s v${argv.version} -m "v${argv.version}"`);
  }
}

main(process.argv);
