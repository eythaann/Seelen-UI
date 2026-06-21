// Custom esbuild plugins

import type esbuild from "esbuild";
import fs, { promises as fsp } from "fs";
import path from "path";

async function copyDirIncremental(src: string, dest: string): Promise<void> {
  await fsp.mkdir(dest, { recursive: true });
  const entries = await fsp.readdir(src, { withFileTypes: true });
  await Promise.all(
    entries.map(async (entry) => {
      const srcPath = path.join(src, entry.name);
      const destPath = path.join(dest, entry.name);
      if (entry.isDirectory()) {
        await copyDirIncremental(srcPath, destPath);
      } else {
        const srcMtime = (await fsp.stat(srcPath)).mtimeMs;
        const destMtime = await fsp.stat(destPath).then((s) => s.mtimeMs).catch(() => 0);
        if (srcMtime > destMtime) {
          await fsp.copyFile(srcPath, destPath);
        }
      }
    }),
  );
}

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
      build.onEnd(async () => {
        try {
          await Promise.all(
            appFolders.map(async (folder) => {
              const copies: Promise<void>[] = [];

              const publicSrc = `src/ui/${folder}/public`;
              if (fs.existsSync(publicSrc)) {
                copies.push(copyDirIncremental(publicSrc, `dist/${folder}`));
              }

              // Eythan: I did this to avoid doing a pr moving all the translations files to public folders.
              const translationsSrc = `src/ui/${folder}/i18n/translations`;
              if (fs.existsSync(translationsSrc)) {
                copies.push(copyDirIncremental(translationsSrc, `dist/${folder}/translations`));
              }

              await Promise.all(copies);
            }),
          );

          // Move nested folders to root
          const distSrcPath = "dist/src/ui";
          if (fs.existsSync(distSrcPath)) {
            const nested = await fsp.readdir(distSrcPath);
            await Promise.all(
              nested.map((folder) => copyDirIncremental(`dist/src/ui/${folder}`, `dist/${folder}`)),
            );
            await fsp.rm("dist/src", { recursive: true, force: true });
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
