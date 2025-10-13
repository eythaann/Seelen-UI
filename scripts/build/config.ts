// Shared build configuration

import yargs from "yargs";
import { hideBin } from "yargs/helpers";
import process from "node:process";
import type { BuildArgs } from "./types.ts";

export async function parseArgs(): Promise<BuildArgs> {
  const argv = await yargs(hideBin(process.argv))
    .option("production", {
      type: "boolean",
      description: "Enable Production Minified Bundle",
      default: false,
    })
    .option("serve", {
      type: "boolean",
      description: "Run a local server",
      default: false,
    }).argv;

  return {
    isProd: !!argv.production,
    serve: !!argv.serve,
  };
}

export const DEV_SERVER_PORT = 3579;
export const DIST_DIR = "./dist";
export const ICONS_DIR = "./dist/icons";
export const UI_DIR = "src/ui";
export const SHARED_DIR = "shared";
export const NODE_MODULES_DIR = "./node_modules";
