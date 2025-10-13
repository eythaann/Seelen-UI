// React/Preact build configuration

import esbuild from "esbuild";
import CssModulesPlugin from "esbuild-css-modules-plugin";
import { createCopyPublicPlugin } from "../plugins/index.ts";
import type { BuildArgs } from "../types.ts";
import { DIST_DIR, NODE_MODULES_DIR, UI_DIR } from "../config.ts";

/**
 * Creates esbuild configuration for React/Preact applications
 * Uses Preact as a drop-in replacement for React for better performance
 */
export function createReactBuildConfig(
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
    outdir: DIST_DIR,
    outbase: `./${UI_DIR}`,
    jsx: "automatic",
    loader: {
      ".yml": "text",
    },
    plugins: [
      CssModulesPlugin({
        localsConvention: "camelCase",
        pattern: "do-not-use-on-themes-[local]-[hash]",
      }),
      createCopyPublicPlugin(appFolders),
    ],
    alias: {
      react: `${NODE_MODULES_DIR}/preact/compat/`,
      "react/jsx-runtime": `${NODE_MODULES_DIR}/preact/jsx-runtime`,
      "react-dom": `${NODE_MODULES_DIR}/preact/compat/`,
      "react-dom/*": `${NODE_MODULES_DIR}/preact/compat/*`,
    },
  };
}

/**
 * Builds React/Preact applications using esbuild
 */
export async function buildReact(
  entryPoints: string[],
  appFolders: string[],
  args: BuildArgs,
): Promise<void> {
  if (entryPoints.length === 0) {
    return;
  }

  const startTime = Date.now();
  const config = createReactBuildConfig(entryPoints, appFolders, args);

  if (args.serve) {
    const ctx = await esbuild.context(config);
    await ctx.watch();
    console.info(`✓ React: ${entryPoints.length} apps watching for changes`);
  } else {
    await esbuild.build(config);
    console.info(`✓ React: ${entryPoints.length} apps built (${Date.now() - startTime}ms)`);
  }
}
