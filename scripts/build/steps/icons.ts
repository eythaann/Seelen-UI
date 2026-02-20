// Icon extraction step

import fs from "fs";
import path from "path";
// preact-render-to-string doesn't work here: react-icons emits real React elements
// ($$typeof: Symbol(react.element)), not Preact VNodes, so we need the real React SSR renderer.
import { renderToStaticMarkup } from "real-react-dom/server";
import { ICONS_DIR, NODE_MODULES_DIR } from "../config.ts";

/**
 * Extracts SVG icons from react-icons package and generates TypeScript types
 * This step only runs if the icons directory doesn't exist
 */
export async function extractIcons(): Promise<void> {
  if (fs.existsSync(ICONS_DIR)) {
    console.info("Icons already extracted, skipping...");
    return;
  }

  console.info("Extracting SVG Lazy Icons");
  console.time("Lazy Icons");

  fs.mkdirSync(ICONS_DIR, { recursive: true });

  let tsFile = "// This file is generated on build, do not edit.\nexport type IconName =";
  const reactIconsPath = path.join(NODE_MODULES_DIR, "react-icons");
  const entries = fs.readdirSync(reactIconsPath);

  for (const entry of entries) {
    const entryPath = path.join(reactIconsPath, entry);
    const isDir = fs.statSync(entryPath).isDirectory();

    if (!isDir || entry === "lib") {
      continue;
    }

    console.info(`Extracting icon family: ${entry}`);

    const family = await import(`react-icons/${entry}`);
    for (const [name, ElementConstructor] of Object.entries(family)) {
      if (typeof ElementConstructor !== "function") {
        continue;
      }
      const element = ElementConstructor({ size: "1em" });
      const svg = renderToStaticMarkup(element);
      if (!svg.startsWith("<svg")) {
        throw new Error(`Invalid SVG: ${svg}`);
      }
      fs.writeFileSync(path.join(ICONS_DIR, `${name}.svg`), svg);
    }

    tsFile += `\n  | keyof typeof import("react-icons/${entry}")`;
  }

  tsFile += ";\n";
  fs.writeFileSync("./libs/ui/icons.ts", tsFile);

  console.timeEnd("Lazy Icons");
}
