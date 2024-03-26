import { createSlice } from '@reduxjs/toolkit';

import { SeelenWegMode, SeelenWegState } from '../../../settings/modules/seelenweg/domain';

const initialState: SeelenWegState = {
  enabled: true,
  mode: SeelenWegMode.MIN_CONTENT,
  size: 40,
  zoomSize: 70,
  margin: 8,
  padding: 8,
  spaceBetweenItems: 8,
};

export const SeelenWegSlice = createSlice({
  name: 'seelenweg',
  initialState,
  reducers: {},
});