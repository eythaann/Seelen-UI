// Development server

import express from "express";
import { DEV_SERVER_PORT, DIST_DIR } from "./config.ts";

/**
 * Starts a local development server to serve built assets
 * Serves static files from the dist directory
 */
export function startDevServer(): void {
  const app = express();

  app.use(express.static(DIST_DIR));

  app.listen(DEV_SERVER_PORT, () => {
    console.info(`\nDevelopment server running at http://localhost:${DEV_SERVER_PORT}`);
    console.info("Watching for changes...\n");
  });
}
