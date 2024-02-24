import { execSync } from 'child_process';
import fs from 'fs';

const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf-8'));

let changelogContent = fs.readFileSync('changelog.md', 'utf-8');

changelogContent = changelogContent.replace(/## \[Unreleased\]/g, `## \[Unreleased\]\n## [${packageJson.version}]`);

fs.writeFileSync('changelog.md', changelogContent);

execSync('git add .');
execSync('git commit --amend --no-edit');