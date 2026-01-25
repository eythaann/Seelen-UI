// Vanilla TypeScript build configuration

import esbuild from "esbuild";
import { createCopyPublicPlugin, createLoggerPlugin } from "../plugins/index.ts";
import type { BuildArgs } from "../types.ts";
import { DIST_DIR, UI_DIR } from "../config.ts";

/**
 * Creates esbuild configuration for Vanilla TypeScript applications
 */
export function createVanillaBuildConfig(
  entryPoints: string[],
  appFolders: string[],
  args: BuildArgs,
  isWatchMode: boolean,
): esbuild.BuildOptions {
  return {
    entryPoints,
    bundle: true,
    minify: args.isProd,
    sourcemap: !args.isProd,
    treeShaking: true,
    format: "esm",
    target: "esnext",
    platform: "browser",
    outdir: DIST_DIR,
    outbase: `./${UI_DIR}`,
    loader: {
      ".yml": "text",
      ".svg": "text",
    },
    plugins: [
      createLoggerPlugin("Vanilla", entryPoints.length, isWatchMode),
      createCopyPublicPlugin(appFolders),
    ],
  };
}

/**
 * Builds Vanilla TypeScript applications using esbuild
 */
export async function buildVanilla(
  entryPoints: string[],
  appFolders: string[],
  args: BuildArgs,
): Promise<void> {
  if (entryPoints.length === 0) {
    return;
  }

  const isWatchMode = args.serve;
  const config = createVanillaBuildConfig(entryPoints, appFolders, args, isWatchMode);

  if (isWatchMode) {
    const ctx = await esbuild.context(config);
    await ctx.watch();
  } else {
    await esbuild.build(config);
  }
}
