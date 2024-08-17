import { execSync } from 'child_process';
import fs from 'fs';

let changelogContent = fs.readFileSync('changelog.md', 'utf-8');
changelogContent = changelogContent.replace(/## \[Unreleased\]/g, `## \[Unreleased\]\n## [${process.env.npm_new_version}]`);
fs.writeFileSync('changelog.md', changelogContent);

execSync('git add changelog.md');

let cargoTomlContent = fs.readFileSync('Cargo.toml', 'utf-8');
cargoTomlContent = cargoTomlContent.replace(/^version\s*=\s*".*"/m, `version = "${process.env.npm_new_version}"`);
fs.writeFileSync('Cargo.toml', cargoTomlContent);
execSync('cargo update -p seelen-ui');

execSync('git add Cargo.toml Cargo.lock');
execSync('git commit --amend --no-edit');

// delete tag created by `npm version` and create a new one
execSync(`git tag -d v${process.env.npm_new_version}`);
execSync(`git tag v${process.env.npm_new_version}`);