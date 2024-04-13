import { createSlice } from '@reduxjs/toolkit';

import { SeelenWegSlice as SeelenWegSettingsSlice } from '../../../settings/modules/seelenweg/app';

import { SeelenWegState } from '../../../settings/modules/seelenweg/domain';

const initialState: SeelenWegState = SeelenWegSettingsSlice.getInitialState();

export const SeelenWegSlice = createSlice({
  name: 'seelenweg',
  initialState,
  reducers: {},
});

