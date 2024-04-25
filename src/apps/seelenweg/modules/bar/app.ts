import { parseAsCamel } from '../../../utils/schemas';
import { Seelenweg, SeelenWegSchema } from '../../../utils/schemas/Seelenweg';
import { createSlice } from '@reduxjs/toolkit';

const initialState: Seelenweg = parseAsCamel(SeelenWegSchema, {});

export const SeelenWegSlice = createSlice({
  name: 'seelenweg',
  initialState,
  reducers: {},
});

