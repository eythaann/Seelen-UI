// Cleanup utilities

import fs from "fs";
import { DIST_DIR } from "../config.ts";

/**
 * Wipes the entire dist directory. Only runs on production builds — dev builds
 * skip this so incremental copies can reuse existing artifacts.
 */
export function cleanDist(): void {
  console.info("Cleaning dist...");
  fs.rmSync(DIST_DIR, { recursive: true, force: true });
  fs.mkdirSync(DIST_DIR, { recursive: true });
}
