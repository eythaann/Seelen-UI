import { parseAsCamel } from '../../../shared/schemas';
import { Seelenweg, SeelenWegSchema } from '../../../shared/schemas/Seelenweg';
import { createSlice } from '@reduxjs/toolkit';

const initialState: Seelenweg = parseAsCamel(SeelenWegSchema, {});

export const SeelenWegSlice = createSlice({
  name: 'seelenweg',
  initialState,
  reducers: {},
});

