import { CaseReducerActions, SliceCaseReducers } from '@reduxjs/toolkit';
import { Event, listen } from '@tauri-apps/api/event';
import { Store } from 'redux';

export class TauriReduxExtension<S, C extends SliceCaseReducers<S>, Name extends string> {
  constructor(private store: Store<S>, private actions: CaseReducerActions<C, Name>) {}

  handleAction<K extends keyof C>(action: K, event: Event<any>) {
    const actionObj = this.actions[action](event.payload);
    this.store.dispatch(actionObj);
  }

  /** Actions exposed to Tauri as `redux://{KeyOfActionCreator}` */
  async globalExpose<K extends keyof C>(...x: [K]) {
    let action = x[0];
    if (typeof action !== 'string') {
      return;
    }
    await listen<any>(`redux://${action}`, this.handleAction.bind(this, action));
  }
}
