const fs = require('fs');

const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf-8'));

let changelogContent = fs.readFileSync('changelog.md', 'utf-8');

changelogContent = changelogContent.replace(/\[Unreleased\]/g, `[${packageJson.version}]`);

fs.writeFileSync('changelog.md', changelogContent);