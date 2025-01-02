import { configureStore } from '@reduxjs/toolkit';
import { ConnectedMonitorList, SeelenEvent, Settings } from '@seelen-ui/lib';
import { getCurrentWebview } from '@tauri-apps/api/webview';

import { Actions, RootSlice } from './app';
import { StartThemingTool } from 'src/apps/shared/styles';

export const store = configureStore({
  reducer: RootSlice.reducer,
  middleware(getDefaultMiddleware) {
    return getDefaultMiddleware({
      serializableCheck: false,
    });
  },
});

export async function initStore() {
  const webview = getCurrentWebview();
  const settings = await Settings.getAsync();

  store.dispatch(Actions.setSettings(settings.inner.wall));
  Settings.onChange((settings) => {
    store.dispatch(Actions.setSettings(settings.inner.wall));
  });

  store.dispatch(Actions.setMonitors((await ConnectedMonitorList.getAsync()).all()));
  ConnectedMonitorList.onChange((monitors: ConnectedMonitorList) => {
    store.dispatch(Actions.setMonitors(monitors.all()));
  });

  webview.listen<boolean>(SeelenEvent.WallStop, ({ payload }) => {
    store.dispatch(Actions.setStop(payload));
  });

  webview.listen(SeelenEvent.SystemMonitorsChanged, () => {
    let version = store.getState().version;
    store.dispatch(Actions.setVersion(version + 1));
  });

  StartThemingTool();
}
