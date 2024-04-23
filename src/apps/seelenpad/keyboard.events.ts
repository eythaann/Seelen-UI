import { throttle } from '../utils/Timing';
import { invoke } from '@tauri-apps/api/core';
import { getCurrent } from '@tauri-apps/api/webviewWindow';

import { store } from './modules/shared/store/infra';
import { playSound } from './modules/shared/utils/infra';

import { RouletteActions, SelectDisplayingRoulette } from './modules/roulette/app/slice';
import { Selectors } from './modules/shared/store/app';

export function registerKeyboarEvents() {
  window.addEventListener('blur', () => {
    console.trace('closing seelenpad by blur event');
    getCurrent().close();
  });

  window.addEventListener(
    'keydown',
    throttle((event) => {
      if (['ArrowRight', 'ArrowUp'].includes(event.key)) {
        const rotationStep = Selectors.roulette.rotationStep(store.getState());
        playSound('pops/pop2.mp3', 0.02);
        store.dispatch(RouletteActions.setRotationStep(rotationStep + 1));
      }

      if (['ArrowLeft', 'ArrowDown'].includes(event.key)) {
        const rotationStep = Selectors.roulette.rotationStep(store.getState());
        playSound('pops/pop2.mp3', 0.02);
        store.dispatch(RouletteActions.setRotationStep(rotationStep - 1));
      }

      if (event.key == 'Escape') {
        getCurrent().close();
      }

      if (event.key == 'Backspace') {
        const state = store.getState();
        const roulette = SelectDisplayingRoulette(state);

        playSound('pops/bloop.mp3');
        store.dispatch(RouletteActions.consumeStack());
        store.dispatch(RouletteActions.setRotationStep(roulette.parentIdx || 0));
      }

      if (event.key == 'Enter') {
        const state = store.getState();
        const rotationStep = Selectors.roulette.rotationStep(state);
        const roulette = SelectDisplayingRoulette(state);

        const selectedItem = roulette.items.at(rotationStep % roulette.items.length);

        if (selectedItem && selectedItem.action) {
          invoke(selectedItem.action).catch((error) => console.error(error));
        }

        if (selectedItem && selectedItem.subItems) {
          playSound('pops/bloop.mp3');
          store.dispatch(RouletteActions.setRotationStep(0));
          store.dispatch(RouletteActions.addToStack({
            parentIdx: selectedItem.position,
            items: selectedItem.subItems,
          }));
        }
      }
    }, 100),
  );
}
