import { RouteExtraInfo, RouteLabels } from '../navigation/routes';
import { getCurrent } from '@tauri-apps/api/webviewWindow';
import { Button, Tooltip } from 'antd';

import { LoadSettingsToStore, SaveStore } from '../../modules/shared/store/infra';
import { useAppSelector } from '../../modules/shared/utils/infra';

import { RootSelectors } from '../../modules/shared/store/app/selectors';

import cs from './index.module.css';

export const Header = () => {
  let route = useAppSelector(RootSelectors.route);
  let hasChanges = useAppSelector(RootSelectors.toBeSaved);

  const SaveOrQuit = () => {
    if (hasChanges) {
      SaveStore();
    } else {
      getCurrent().close();
    }
  };

  return (
    <div className={cs.Header} data-tauri-drag-region>
      <div>
        {RouteLabels[route]}{' '}
        {RouteExtraInfo[route] && (
          <Tooltip title={RouteExtraInfo[route]}>
            <span className={cs.info}>🛈</span>
          </Tooltip>
        )}
      </div>
      <div>
        <Button children="Cancel" type="default" danger disabled={!hasChanges} onClick={() => LoadSettingsToStore()} />
        {'  '}
        <Button children={hasChanges ? 'Save' : 'Close'} type="primary" danger={!hasChanges} onClick={SaveOrQuit} />
      </div>
    </div>
  );
};
