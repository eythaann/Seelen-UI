import { RootSlice } from './reducer';

import { BorderSlice } from '../../general/border/app';
import { ContainerTopBarSlice } from '../../general/containerTopBar/app';
import { GeneralSettingsSlice } from '../../general/main/app';
import { PopupSlice } from '../../general/popups/app';
import { SeelenWegSlice } from '../../seelenweg/app';

import { RootState } from '../domain/state';

export const ownSelector = (state: RootState) => state;
export const RootSelectors = RootSlice.getSelectors(ownSelector);

export const GeneralSettingsSelectors = GeneralSettingsSlice.getSelectors(RootSelectors.generals);
export const SeelenWegSelectors = SeelenWegSlice.getSelectors(RootSelectors.seelenweg);

export const BorderSelectors = BorderSlice.getSelectors(GeneralSettingsSelectors.border);
export const PopupSelectors = PopupSlice.getSelectors(GeneralSettingsSelectors.popups);
export const ContainerTopBarSelectors = ContainerTopBarSlice.getSelectors(GeneralSettingsSelectors.containerTopBar);

export const getMonitorSelector = (idx: number) => (state: RootState) => RootSelectors.monitors(state)[idx];
export const getWorkspaceSelector = (idx: number, monitorIdx: number) => (state: RootState) => getMonitorSelector(monitorIdx)(state)?.workspaces[idx];
