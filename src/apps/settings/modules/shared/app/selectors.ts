import { RootSlice } from './reducer';

import { GeneralSettingsSlice } from '../../general/main/app';
import { SeelenWegSlice } from '../../seelenweg/app';
import { BorderSlice } from '../../WindowManager/border/app';
import { ContainerTopBarSlice } from '../../WindowManager/containerTopBar/app';
import { SeelenManagerSlice } from '../../WindowManager/main/app';

import { RootState } from '../domain/state';

export const ownSelector = (state: RootState) => state;
export const RootSelectors = RootSlice.getSelectors(ownSelector);

export const GeneralSettingsSelectors = GeneralSettingsSlice.getSelectors(RootSelectors.generals);
export const SeelenWegSelectors = SeelenWegSlice.getSelectors(RootSelectors.seelenweg);
export const SeelenWmSelectors = SeelenManagerSlice.getSelectors(RootSelectors.seelenwm);

export const BorderSelectors = BorderSlice.getSelectors(SeelenWmSelectors.border);
export const ContainerTopBarSelectors = ContainerTopBarSlice.getSelectors(SeelenWmSelectors.containerTopBar);

export const getMonitorSelector = (idx: number) => (state: RootState) => RootSelectors.monitors(state)[idx];
export const getWorkspaceSelector = (idx: number, monitorIdx: number) => (state: RootState) => getMonitorSelector(monitorIdx)(state)?.workspaces[idx];
