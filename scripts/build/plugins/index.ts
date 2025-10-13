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
