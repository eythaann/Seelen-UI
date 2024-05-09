import { execSync } from 'child_process';
import fs from 'fs';

let changelogContent = fs.readFileSync('changelog.md', 'utf-8');
changelogContent = changelogContent.replace(/## \[Unreleased\]/g, `## \[Unreleased\]\n## [${process.env.npm_package_version}]`);
fs.writeFileSync('changelog.md', changelogContent);

execSync('git add .');
execSync('git commit --amend --no-edit');