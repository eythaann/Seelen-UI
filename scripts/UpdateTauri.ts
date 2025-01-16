import { execSync } from 'child_process';
import { readFileSync } from 'fs';
import toml from 'toml';

import packageJson from '../package.json';

let dependencies = { ...packageJson.dependencies, ...packageJson.devDependencies };
let toUpdate: string[] = [];

for (let key in dependencies) {
  if (key.startsWith('@tauri-apps/')) {
    toUpdate.push(key);
  }
}

let command = `npm update ${toUpdate.join(' ')}`;
console.info(`${command}\n`);
execSync(command, { stdio: 'inherit' });

const cargoToml = toml.parse(readFileSync('Cargo.toml', 'utf-8'));
dependencies = { ...cargoToml['build-dependencies'], ...cargoToml.dependencies };
toUpdate = [];

for (let key in dependencies) {
  if (key.startsWith('tauri')) {
    toUpdate.push(key);
  }
}

command = `cargo update ${toUpdate.join(' ')}`;
console.info(`${command}\n`);
execSync(command, { stdio: 'inherit' });
