// Svelte build configuration

import esbuild from "esbuild";
import { createCopyPublicPlugin } from "../plugins/index.ts";
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
): esbuild.BuildOptions {
  return {
    entryPoints,
    bundle: true,
    minify: args.isProd,
    sourcemap: !args.isProd,
    treeShaking: true,
    format: "esm",
    target: "esnext",
    outdir: DIST_DIR,
    outbase: `./${UI_DIR}`,
    loader: {
      ".yml": "text",
    },
    plugins: [
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

  const startTime = Date.now();
  const config = createSvelteBuildConfig(entryPoints, appFolders, args);

  if (args.serve) {
    const ctx = await esbuild.context(config);
    await ctx.watch();
    console.info(`✓ Svelte: ${entryPoints.length} apps watching for changes`);
  } else {
    await esbuild.build(config);
    console.info(`✓ Svelte: ${entryPoints.length} apps built (${Date.now() - startTime}ms)`);
  }
}
