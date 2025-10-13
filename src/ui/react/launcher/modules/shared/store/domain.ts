import type { SeelenLauncherSettings } from "@seelen-ui/lib/types";
import type { IRootState } from "src/ui/reduxRootState";

export interface StartMenuApp {
  path: string;
  umid: string | null;
  target: string | null;
}

export interface LauncherState extends IRootState<SeelenLauncherSettings> {
  apps: StartMenuApp[];
  history: { [key in string]?: string[] };
}
