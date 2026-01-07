// Main build orchestrator
// This file coordinates the build process for Seelen UI applications

import { parseArgs } from "./build/config.ts";
import { extractIcons } from "./build/steps/icons.ts";
import { cleanDist } from "./build/steps/cleanup.ts";
import { discoverEntryPoints, groupEntryPointsByFramework } from "./build/steps/discover.ts";
import { buildReact } from "./build/builders/react.ts";
import { buildSvelte } from "./build/builders/svelte.ts";
import { buildVanilla } from "./build/builders/vanilla.ts";
import { startDevServer } from "./build/server.ts";
import process from "node:process";

/**
 * Main build function
 * Orchestrates the entire build process:
 * 1. Parse command-line arguments
 * 2. Extract icons (in parallel with cleaning)
 * 3. Clean dist directory (in parallel with icons)
 * 4. Discover entry points
 * 5. Build all frameworks in parallel (React, Svelte, Vanilla)
 * 6. Start dev server (if --serve flag is set) - serves the entire dist folder
 */
async function main() {
  const args = await parseArgs();
  console.info(`Build mode: ${args.isProd ? "production" : "development"}`);
  console.info(`Serve: ${args.serve ? "enabled" : "disabled"}\n`);

  // Step 1 & 2: Extract icons and clean dist directory in parallel
  await Promise.all([extractIcons(), Promise.resolve(cleanDist())]);

  // Step 3: Discover entry points
  const entryPoints = discoverEntryPoints();
  const groupedEntryPoints = groupEntryPointsByFramework(entryPoints);

  console.info(`\nDiscovered entry points:`);
  console.info(`  React: ${groupedEntryPoints.react.length}`);
  console.info(`  Svelte: ${groupedEntryPoints.svelte.length}`);
  console.info(`  Vanilla: ${groupedEntryPoints.vanilla.length}`);
  console.info();

  // Collect all app folders for public file copying
  const allAppFolders = entryPoints.map((entry) => entry.folder);

  // Step 4: Build all frameworks in parallel
  console.time("Total build time");

  await Promise.all([
    buildReact(groupedEntryPoints.react, allAppFolders, args),
    buildSvelte(groupedEntryPoints.svelte, allAppFolders, args),
    buildVanilla(groupedEntryPoints.vanilla, allAppFolders, args),
  ]);

  console.timeEnd("Total build time");

  // Step 5: Start dev server if requested (serves entire dist folder)
  if (args.serve) {
    startDevServer();
  }

  console.info("\n✓ Build complete!\n");
}

// Run the build
main().catch((error) => {
  console.error("Build failed:", error);
  process.exit(1);
});
