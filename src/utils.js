const { exec } = require('child_process');
const { app } = require('electron');
const { readFileSync, writeFileSync } = require('fs');
const os = require('os');
const path = require('path');

const fromCurrent = (...segments) => {
  return path.join(__dirname, ...segments);
};

const fromPackage = (...segments) => {
  return path.join(app.getAppPath(), '../../', ...segments);
};

const execPrinter = (error, stdout, stderr) => {
  if (error) {
    console.error(`Error: ${error.message}`);
  }
  if (stderr) {
    console.error(`STDERR: ${stderr}`);
  }
  if (stdout) {
    console.log(`STDOUT: ${stdout}`);
  }
};

const runPwshScript = (name, args = '') => {
  const tempRoute = path.join(os.tmpdir(), `${Math.random()}-komorebi.ps1`.slice(2));
  writeFileSync(tempRoute, readFileSync(fromCurrent('./pwsh', name)).toString());
  exec(`powershell -ExecutionPolicy Bypass -File ${tempRoute} ${args}`, execPrinter);
};

module.exports = { fromCurrent, runPwshScript, fromPackage };