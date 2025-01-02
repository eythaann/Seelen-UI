import { ConnectedMonitor } from '@seelen-ui/lib';
import { SeelenWallSettings } from '@seelen-ui/lib/types';
import { IRootState } from 'src/shared.interfaces';

export interface RootState extends IRootState<SeelenWallSettings> {
  stop: boolean;
  version: number;
  monitors: ConnectedMonitor[];
}
