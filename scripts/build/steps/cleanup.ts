// Cleanup utilities

import fs from "fs";
import path from "path";
import { DIST_DIR } from "../config.ts";

/**
 * Cleans the dist directory, preserving the icons folder
 * Removes all files and folders except the icons directory
 */
export function cleanDist(): void {
  console.info("Cleaning old artifacts...");

  if (!fs.existsSync(DIST_DIR)) {
    fs.mkdirSync(DIST_DIR, { recursive: true });
    return;
  }

  const entries = fs.readdirSync(DIST_DIR);

  for (const entry of entries) {
    const entryPath = path.join(DIST_DIR, entry);

    // Preserve icons directory
    if (entry === "icons") {
      continue;
    }

    fs.rmSync(entryPath, { recursive: true, force: true });
  }
}
