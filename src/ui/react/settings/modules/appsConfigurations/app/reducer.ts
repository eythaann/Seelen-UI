import type { AppConfig } from "@seelen-ui/lib/types";
import { appsConfig, settings } from "../../../state/mod";

interface AppPayload {
  idx: number;
}

function setConfigByApps(newState: AppConfig[]) {
  settings.value = {
    ...settings.value,
    byApp: newState.filter((app) => !app.isBundled),
  };
}

export const actions = {
  delete: (payload: number) => {
    const newState = [...appsConfig.value].filter((_, idx) => idx !== payload);
    setConfigByApps(newState);
  },

  deleteMany: (payload: number[]) => {
    const newState = [...appsConfig.value].filter((_, idx) => !payload.includes(idx));
    setConfigByApps(newState);
  },

  push: (payload: AppConfig[]) => {
    setConfigByApps([...appsConfig.value, ...payload]);
  },

  replace: (payload: AppPayload & { app: AppConfig }) => {
    const { idx, app } = payload;
    const newState = [...appsConfig.value];
    newState[idx] = app;
    setConfigByApps(newState);
  },

  swap: (payload: [number, number]) => {
    const [idx1, idx2] = payload;
    const App1 = appsConfig.value[idx1];
    const App2 = appsConfig.value[idx2];

    const newState = [...appsConfig.value];
    if (App1 && App2) {
      newState[idx1] = App2;
      newState[idx2] = App1;
    }

    setConfigByApps(newState);
  },
};
