import { execSync } from 'child_process';
import fs from 'fs';

let changelogContent = fs.readFileSync('changelog.md', 'utf-8');
changelogContent = changelogContent.replace(/## \[Unreleased\]/g, `## \[Unreleased\]\n## [${process.env.npm_new_version}]`);
fs.writeFileSync('changelog.md', changelogContent);

execSync('git add changelog.md');
execSync(`git commit -m "v${process.env.npm_new_version}: update changelog"`);