import { useSelector } from 'react-redux';

import { Selectors } from '../shared/store/app';

export function MediaItem() {
  const sessions = useSelector(Selectors.mediaSessions);
  const defaultSession = sessions.find((s) => s.default) || sessions[0];

  return <div>
    <div>{defaultSession?.title || 'No Media'}</div>
    <div>{defaultSession?.author || ''}</div>
  </div>;
}