import { getWorkspaceSelector, SeelenWmSelectors } from '../../shared/store/app/selectors';
import { defaultOnNull } from '../../shared/utils/app';

import { RootState } from '../../shared/store/domain';

export const getWorkspacePaddingSelector = (idx: number, monitorIdx: number) => (state: RootState) => {
  return defaultOnNull(
    getWorkspaceSelector(idx, monitorIdx)(state)?.workspacePadding,
    SeelenWmSelectors.workspacePadding(state),
  );
};

export const getContainerPaddingSelector = (idx: number, monitorIdx: number) => (state: RootState) => {
  return defaultOnNull(
    getWorkspaceSelector(idx, monitorIdx)(state)?.containerPadding,
    SeelenWmSelectors.containerPadding(state),
  );
};
