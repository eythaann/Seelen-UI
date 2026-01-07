// Custom esbuild plugins

import type esbuild from "esbuild";
import fs from "fs";

/**
 * Custom plugin to handle post-build operations:
 * - Copies public folders for each widget
 * - Moves nested dist folders to root
 * - Cleans up temporary directories
 */
export function createCopyPublicPlugin(appFolders: string[]): esbuild.Plugin {
  return {
    name: "copy-public-by-entry",
    setup(build) {
      build.onEnd(() => {
        try {
          // Copy public folder for each widget
          for (const folder of appFolders) {
            const source = `src/ui/${folder}/public`;
            const target = `dist/${folder}`;

            if (fs.existsSync(source)) {
              fs.cpSync(source, target, { recursive: true, force: true });
            }
          }

          // Move nested folders to root
          const distSrcPath = "dist/src/ui";
          if (fs.existsSync(distSrcPath)) {
            for (const folder of fs.readdirSync(distSrcPath)) {
              const source = `dist/src/ui/${folder}`;
              const target = `dist/${folder}`;
              fs.cpSync(source, target, { recursive: true, force: true });
            }
            fs.rmSync("dist/src", { recursive: true, force: true });
          }
        } catch (error) {
          console.error("Error in copy-public-by-entry plugin:", error);
        }
      });
    },
  };
}

/**
 * Logger plugin to track build lifecycle events
 * - Logs build start with framework name
 * - Logs build completion with timing and result status
 * - Logs errors and warnings if any occur
 */
export function createLoggerPlugin(
  frameworkName: string,
  entryPointsCount: number,
  isWatchMode: boolean,
): esbuild.Plugin {
  let startTime = 0;

  return {
    name: "logger",
    setup(build) {
      build.onStart(() => {
        startTime = Date.now();
        const mode = isWatchMode ? "watch" : "build";
        console.info(
          `⚙  ${frameworkName}: Starting ${mode} for ${entryPointsCount} app${entryPointsCount !== 1 ? "s" : ""}...`,
        );
      });

      build.onEnd((result) => {
        const duration = Date.now() - startTime;
        const hasErrors = result.errors.length > 0;
        const hasWarnings = result.warnings.length > 0;

        if (hasErrors) {
          console.error(
            `✗ ${frameworkName}: Build failed with ${result.errors.length} error${
              result.errors.length !== 1 ? "s" : ""
            } (${duration}ms)`,
          );
          return;
        }

        if (hasWarnings) {
          console.warn(
            `⚠ ${frameworkName}: Built with ${result.warnings.length} warning${
              result.warnings.length !== 1 ? "s" : ""
            } (${duration}ms)`,
          );
        }

        if (isWatchMode) {
          console.info(`✓ ${frameworkName}: Watching for changes (${duration}ms)`);
        } else {
          console.info(`✓ ${frameworkName}: Build completed (${duration}ms)`);
        }
      });
    },
  };
}
