import { SeelenWegState } from '../../../utils/interfaces/Weg';
import { createSlice } from '@reduxjs/toolkit';

import { SeelenWegSlice as SeelenWegSettingsSlice } from '../../../settings/modules/seelenweg/app';

const initialState: SeelenWegState = SeelenWegSettingsSlice.getInitialState();

export const SeelenWegSlice = createSlice({
  name: 'seelenweg',
  initialState,
  reducers: {},
});

