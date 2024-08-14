import { execSync } from 'child_process';
import fs from 'fs';

let changelogContent = fs.readFileSync('changelog.md', 'utf-8');
changelogContent = changelogContent.replace(/## \[Unreleased\]/g, `## \[Unreleased\]\n## [${process.env.npm_new_version}]`);
fs.writeFileSync('changelog.md', changelogContent);

execSync('git add changelog.md');

let cargoTomlContent = fs.readFileSync('Cargo.toml', 'utf-8');
cargoTomlContent = cargoTomlContent.replace(/^version\s*=\s*".*"/m, `version = "${process.env.npm_new_version}"`);
fs.writeFileSync('Cargo.toml', cargoTomlContent);

execSync('git add Cargo.toml');
execSync('git commit --amend --no-edit');