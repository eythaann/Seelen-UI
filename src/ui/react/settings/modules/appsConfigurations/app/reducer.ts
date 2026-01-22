import type { AppConfig } from "@seelen-ui/lib/types";
import { appsConfigs } from "../../../state/resources";

interface AppPayload {
  idx: number;
}

export const actions = {
  delete: (payload: number) => {
    const newState = [...appsConfigs.value];
    appsConfigs.value = newState.filter((_, idx) => idx !== payload);
  },
  deleteMany: (payload: number[]) => {
    const newState = [...appsConfigs.value];
    appsConfigs.value = newState.filter((_, idx) => !payload.includes(idx));
  },
  push: (payload: AppConfig[]) => {
    appsConfigs.value = [...appsConfigs.value, ...payload];
  },
  replace: (payload: AppPayload & { app: AppConfig }) => {
    const { idx, app } = payload;
    const newState = [...appsConfigs.value];
    newState[idx] = app;
    appsConfigs.value = newState;
  },
  swap: (payload: [number, number]) => {
    const [idx1, idx2] = payload;
    const App1 = appsConfigs.value[idx1];
    const App2 = appsConfigs.value[idx2];

    const newState = [...appsConfigs.value];
    if (App1 && App2) {
      newState[idx1] = App2;
      newState[idx2] = App1;
    }
    appsConfigs.value = newState;
  },
};
