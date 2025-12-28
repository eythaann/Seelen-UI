// Entry points discovery

import fs from "fs";
import path from "path";
import { UI_DIR } from "../config.ts";
import type { EntryPointInfo, FrameworkType } from "../types.ts";

/**
 * Supported framework folders
 * These are the expected framework directories in the UI folder
 */
export const FRAMEWORK_FOLDERS = ["react", "svelte", "vanilla"] as const;

/**
 * Discovers all application folders within a framework folder
 */
export function discoverAppFolders(frameworkFolder: string): string[] {
  const frameworkPath = path.join(UI_DIR, frameworkFolder);

  if (!fs.existsSync(frameworkPath)) {
    return [];
  }

  return fs
    .readdirSync(frameworkPath)
    .filter((item) => {
      const itemPath = path.join(frameworkPath, item);
      return fs.statSync(itemPath).isDirectory();
    })
    .map((appFolder) => path.join(frameworkFolder, appFolder));
}

/**
 * Discovers entry points for each framework
 * Checks for vanilla (.ts), react (.tsx), and svelte (.svelte) entry points
 */
export function discoverEntryPoints(): EntryPointInfo[] {
  const entryPoints: EntryPointInfo[] = [];

  for (const frameworkFolder of FRAMEWORK_FOLDERS) {
    const appFolders = discoverAppFolders(frameworkFolder);

    for (const appFolder of appFolders) {
      const tsxPath = `./src/ui/${appFolder}/index.tsx`;
      const tsPath = `./src/ui/${appFolder}/index.ts`;
      const sveltePath = `./src/ui/${appFolder}/index.ts`;

      let framework: FrameworkType;
      let entryPath: string;

      // Determine framework based on folder name and file extension
      if (frameworkFolder === "react" && fs.existsSync(tsxPath)) {
        framework = "react";
        entryPath = tsxPath;
      } else if (frameworkFolder === "svelte" && fs.existsSync(sveltePath)) {
        framework = "svelte";
        entryPath = sveltePath;
      } else if (frameworkFolder === "vanilla" && fs.existsSync(tsPath)) {
        framework = "vanilla";
        entryPath = tsPath;
      } else {
        continue; // Skip if no valid entry point found
      }

      entryPoints.push({
        path: entryPath,
        framework,
        folder: appFolder,
      });
    }
  }

  return entryPoints;
}

/**
 * Groups entry points by framework type
 */
export function groupEntryPointsByFramework(
  entryPoints: EntryPointInfo[],
): Record<FrameworkType, string[]> {
  const grouped: Record<FrameworkType, string[]> = {
    react: [],
    svelte: [],
    vanilla: [],
  };

  for (const entry of entryPoints) {
    grouped[entry.framework].push(entry.path);
  }

  return grouped;
}
