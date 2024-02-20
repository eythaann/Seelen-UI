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
    console.error(`\nError: ${error.message}`);
  }
  if (stderr) {
    console.error(`\nSTDERR: ${stderr}\n`);
  }
  if (stdout) {
    console.log(`\nSTDOUT: ${stdout}`);
  }
};

const runPwshScript = async (name, args = '') => {
  const tempRoute = path.join(os.tmpdir(), `${Math.random()}-komorebi.ps1`.slice(2));
  writeFileSync(tempRoute, readFileSync(fromCurrent('./pwsh', name)).toString());
  return new Promise((resolve, reject) => {
    exec(`powershell -ExecutionPolicy Bypass -File ${tempRoute} ${args}`, (error, stdout, stderr) => {
      execPrinter(error, stdout, stderr);
      if (error) {
        return reject(error);
      }
      resolve();
    });
  });
};

module.exports = { fromCurrent, runPwshScript, fromPackage };