import { AppIdentifierType, MatchingStrategy } from "@seelen-ui/lib";
import { AppConfig } from "@seelen-ui/lib/types";

export const defaultAppConfig: AppConfig = {
  name: "New App",
  identifier: {
    id: "new-app.exe",
    matchingStrategy: MatchingStrategy.Equals,
    kind: AppIdentifierType.Exe,
    negation: false,
    or: [],
    and: [],
  },
  options: [],
  isBundled: false,
  category: null,
  boundMonitor: null,
  boundWorkspace: null,
};
