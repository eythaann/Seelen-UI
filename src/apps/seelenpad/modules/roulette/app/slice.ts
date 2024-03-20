import { createSlice, PayloadAction } from '@reduxjs/toolkit';

import { reducersFor } from '../../../../settings/modules/shared/app/utils';
import { Selectors } from '../../shared/store/app';

import { RootState } from '../../shared/store/domain';
import { Item, RouletteStackItem, RouletteState } from '../domain';

const itemsRaw: Item[] = [
  {
    label: 'Media',
    icon: '🎵',
    position: 0,
    subItems: [
      {
        label: 'play/pause',
        icon: '⏯️',
        position: 0,
        action: 'media_play_pause',
      },
      {
        label: 'next',
        icon: '⏭️',
        position: 1,
        action: 'media_next',
      },
      {
        label: 'play/pause',
        icon: '⏯️',
        position: 2,
        action: 'media_play_pause',
      },
      {
        label: 'prev',
        icon: '⏮️',
        position: 3,
        action: 'media_prev',
      },
    ],
  },
];

let icons = ['❤️‍🔥', '🦀', '😄', '🤍', '🤔', '🐛', '💀', '🌅', '👁️', '🎨', '💻', '🎮'];

for (let i = 0; i < 5; i++) {
  const subItems: Item[] = [];
  for (let j = 0; j < 4; j++) {
    subItems.push({
      label: 'Subitem ' + i + '-' + j,
      icon: icons[j]!,
      position: j,
    });
  }

  itemsRaw.push({
    label: 'Sponsor Project ' + i,
    icon: icons[i]!,
    subItems,
    position: i + 1,
  });
}

const initialState: RouletteState = {
  stack: [{ items: itemsRaw }],
  rotationStep: 0,
};

export const RouletteSlice = createSlice({
  name: 'roulette',
  initialState,
  reducers: {
    consumeStack(state) {
      if (state.stack.length > 1) {
        state.stack.shift();
      }
    },
    addToStack(state, action: PayloadAction<RouletteStackItem>) {
      state.stack.unshift(action.payload);
    },
    ...reducersFor(initialState),
  },
});

export const RouletteActions = RouletteSlice.actions;

export const SelectDisplayingRoulette = (state: RootState) => Selectors.roulette.stack(state)[0]!;
