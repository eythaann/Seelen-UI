import { WindowManagerSettings } from '@seelen-ui/lib/types';

import { Reservation } from '../../layout/domain';

import { IRootState } from '../../../../reduxRootState';

export interface RootState extends IRootState<WindowManagerSettings> {
  reservation: Reservation | null;
}
