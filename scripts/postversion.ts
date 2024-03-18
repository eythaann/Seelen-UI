import { execSync } from 'child_process';
import fs from 'fs';

const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf-8'));

let changelogContent = fs.readFileSync('changelog.md', 'utf-8');

const regex = /(?<=\[Unreleased\]\s)([\s\S]*?)(?=\s## \[)/g;
const releaseNotes = changelogContent.match(regex)?.[0].trim() || '';

const variables = {
  version: packageJson.version,
  notes: releaseNotes.replaceAll(/\r/g, '').replaceAll(/\n/g, '\\n'),
  current_date: new Date().toISOString(),
  signature: fs.readFileSync(`target/release/bundle/nsis/Komorebi UI_${packageJson.version}_x64-setup.nsis.zip.sig`, 'utf-8'),
};
const jsonString = fs.readFileSync('templates/release.json', 'utf-8');
const replacedString = jsonString.replace(/{{(.*?)}}/g, (match, p1 ) => {
  return (variables as any)[p1] || match;
});
fs.writeFileSync('release', replacedString);

changelogContent = changelogContent.replace(/## \[Unreleased\]/g, `## \[Unreleased\]\n## [${packageJson.version}]`);
fs.writeFileSync('changelog.md', changelogContent);

execSync('git add .');
execSync('git commit --amend --no-edit');

execSync('npm run build');