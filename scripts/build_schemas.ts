import { execSync } from 'child_process';
import fs from 'fs';

(async function main() {
  execSync('cd ./lib && cargo run');
  fs.cpSync('lib/dist', 'documentation/schemas', { recursive: true });
})();
