import { GlobalState } from '../domain/state';

export const selectRoute = (state: GlobalState) => state.route;