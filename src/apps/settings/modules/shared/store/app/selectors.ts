import { SeelenWegSlice } from '../../../seelenweg/app';
import { BorderSlice } from '../../../WindowManager/border/app';
import { SeelenManagerSlice } from '../../../WindowManager/main/app';

import { RootState } from '../domain';

import { RootSlice } from './reducer';

export const ownSelector = (state: RootState) => state;
export const RootSelectors = RootSlice.getSelectors(ownSelector);

export const SeelenWegSelectors = SeelenWegSlice.getSelectors(RootSelectors.seelenweg);
export const SeelenWmSelectors = SeelenManagerSlice.getSelectors(RootSelectors.windowManager);

export const BorderSelectors = BorderSlice.getSelectors(SeelenWmSelectors.border);

export const getMonitorSelector = (idx: number) => (state: RootState) => RootSelectors.monitors(state)[idx];
export const getWorkspaceSelector = (idx: number, monitorIdx: number) => (state: RootState) => getMonitorSelector(monitorIdx)(state)?.workspaces[idx];
