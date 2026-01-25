// Svelte build configuration

import esbuild from "esbuild";
import { createCopyPublicPlugin, createLoggerPlugin } from "../plugins/index.ts";
import type { BuildArgs } from "../types.ts";
import sveltePlugin from "esbuild-svelte";
import { sveltePreprocess } from "svelte-preprocess";
import { DIST_DIR, UI_DIR } from "../config.ts";

/**
 * Creates esbuild configuration for Svelte applications
 * Note: This is a placeholder configuration. You'll need to add the appropriate
 * Svelte esbuild plugin (e.g., esbuild-svelte) to your dependencies and configure it here.
 */
export function createSvelteBuildConfig(
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
    },
    conditions: ["svelte"], // needed to support some imports
    plugins: [
      createLoggerPlugin("Svelte", entryPoints.length, isWatchMode),
      sveltePlugin({
        cache: false,
        preprocess: sveltePreprocess({
          postcss: {
            plugins: [],
          },
        }),
      }),
      createCopyPublicPlugin(appFolders),
    ],
    // Add any Svelte-specific aliases here if needed
    alias: {},
  };
}

/**
 * Builds Svelte applications using esbuild
 */
export async function buildSvelte(
  entryPoints: string[],
  appFolders: string[],
  args: BuildArgs,
): Promise<void> {
  if (entryPoints.length === 0) {
    return;
  }

  const isWatchMode = args.serve;
  const config = createSvelteBuildConfig(entryPoints, appFolders, args, isWatchMode);

  if (isWatchMode) {
    const ctx = await esbuild.context(config);
    await ctx.watch();
  } else {
    await esbuild.build(config);
  }
}
