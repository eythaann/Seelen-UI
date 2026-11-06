// Repeat the opt-in native resource benchmark. Not GPU timing.
import { spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";
const output = process.argv[2];
if (!output) throw new Error("Usage: node scripts/native-blur-native-perf.mjs <result.json>");
const samples = [];
for (let repetition = 0; repetition < (process.argv.includes("--lifecycle-only") ? 0 : 5); repetition++) {
  const run = spawnSync(process.env.CARGO || "cargo", [
    "test",
    "-p",
    "seelen-ui",
    "--bin",
    "seelen-ui",
    "resource_baseline",
    "--quiet",
    "--",
    "--ignored",
    "--nocapture",
    "--test-threads=1",
  ], { encoding: "utf8", env: process.env });
  if (run.status !== 0) throw new Error(run.stdout + run.stderr);
  const results = [...run.stdout.matchAll(/NATIVE_BLUR_PERF (\{[^\n]+\})/g)].map((match) => JSON.parse(match[1]));
  if (results.length !== 3) throw new Error("Expected 1/8/32 native region samples");
  samples.push({ repetition, results });
}
const result = {
  timestamp: new Date().toISOString(),
  profile: "unoptimized debug",
  note: "CPU submission and exact resource-operation counts; not compositor/GPU completion time.",
  samples,
};
if (process.argv.includes("--lifecycle-only")) {
  const run = spawnSync(process.env.CARGO || "cargo", [
    "test",
    "-p",
    "seelen-ui",
    "--bin",
    "seelen-ui",
    "lifecycle_stress",
    "--quiet",
    "--",
    "--ignored",
    "--nocapture",
    "--test-threads=1",
  ], { encoding: "utf8", env: process.env });
  if (run.status !== 0) throw new Error(run.stdout + run.stderr);
  const match = /NATIVE_BLUR_LIFECYCLE (\{[^\n]+\})/.exec(run.stdout);
  if (!match) throw new Error("Missing lifecycle result");
  result.lifecycle = JSON.parse(match[1]);
}
writeFileSync(output, JSON.stringify(result, null, 2) + "\n");
console.log(JSON.stringify(result));
