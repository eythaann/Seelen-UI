import { GlobalState } from '../domain/state';

export const selectRoute = (state: GlobalState) => {
  console.log(state);
  return state.route;
};