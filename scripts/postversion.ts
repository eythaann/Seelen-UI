import { execSync } from 'child_process';
import fs from 'fs';

const newVersion = process.env.npm_new_version;

if (!newVersion) {
  console.error('Missing new version');
  process.exit(1);
}

// update version in Cargo.toml
let cargoTomlContent = fs.readFileSync('Cargo.toml', 'utf-8');
cargoTomlContent = cargoTomlContent.replace(/^version\s*=\s*".*"/m, `version = "${newVersion}"`);
fs.writeFileSync('Cargo.toml', cargoTomlContent);
execSync('cargo update -p seelen-ui');
execSync('git add Cargo.toml Cargo.lock');

// update changelog only on release channel
if (!newVersion.includes('-')) {
  let changelogContent = fs.readFileSync('changelog.md', 'utf-8');
  changelogContent = changelogContent.replace(
    /## \[Unreleased\]/g,
    `## \[Unreleased\]\n## [${newVersion}]`,
  );
  fs.writeFileSync('changelog.md', changelogContent);
  execSync('git add changelog.md');
}

// commit changes
execSync('git commit --amend --no-edit');

// delete tag created by `npm version` and create a new one
execSync(`git tag -d v${newVersion}`);
execSync(`git tag -s v${newVersion} -m "v${newVersion}"`);