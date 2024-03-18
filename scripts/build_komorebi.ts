import { execSync } from 'child_process';
import fs from 'fs';
import { join } from 'path';

const projectPath = './komorebi';

try {
  // Change directory to ./komorebi
  process.chdir(projectPath);

  // Run cargo build command
  execSync('cargo build --locked --release --target x86_64-pc-windows-msvc', { stdio: 'inherit' });

  // Move the compiled executables
  const sourcePath = join('./', 'target', 'x86_64-pc-windows-msvc', 'release');
  fs.copyFileSync(join(sourcePath, 'komorebi.exe'), '../komorebi-wm.exe');
  fs.copyFileSync(join(sourcePath, 'komorebic.exe'), '../komorebic.exe');

  console.log('    Compilation and file movement completed successfully.');
} catch (error: any) {
  console.error('Error:', error);
}