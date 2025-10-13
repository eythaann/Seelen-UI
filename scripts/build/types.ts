// Build configuration types and interfaces

export interface BuildArgs {
  isProd: boolean;
  serve: boolean;
}

export interface BuildContext {
  isProd: boolean;
  serve: boolean;
  appFolders: string[];
  entryPoints: string[];
}

export type FrameworkType = "react" | "svelte" | "vanilla";

export interface EntryPointInfo {
  path: string;
  framework: FrameworkType;
  folder: string;
}
