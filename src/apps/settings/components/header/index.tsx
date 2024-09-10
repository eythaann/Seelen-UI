import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { relaunch } from '@tauri-apps/plugin-process';
import { Button, Tooltip } from 'antd';
import { useTranslation } from 'react-i18next';
import { useDispatch } from 'react-redux';

import { SaveStore } from '../../modules/shared/store/infra';
import { useAppSelector } from '../../modules/shared/utils/infra';

import { RootActions } from '../../modules/shared/store/app/reducer';
import { RootSelectors } from '../../modules/shared/store/app/selectors';

import { RouteExtraInfo } from '../navigation/routes';
import cs from './index.module.css';

export const Header = () => {
  let route = useAppSelector(RootSelectors.route);
  let hasChanges = useAppSelector(RootSelectors.toBeSaved);
  let shouldRestart = useAppSelector(RootSelectors.toBeRestarted);

  const { t } = useTranslation();

  const dispatch = useDispatch();

  const cancelChanges = () => {
    dispatch(RootActions.restoreToLastLoaded());
  };

  const SaveOrQuit = async () => {
    if (hasChanges) {
      await SaveStore();
      if (shouldRestart) {
        await relaunch();
      }
    } else {
      await getCurrentWebviewWindow().close();
    }
  };

  const saveLabel = shouldRestart ? t('save_and_restart') : t('save');
  return (
    <div className={cs.Header} data-tauri-drag-region>
      <div>
        {t(`header.labels.${route}`)}
        {RouteExtraInfo[route] && (
          <Tooltip title={RouteExtraInfo[route]}>
            <span className={cs.info}>🛈</span>
          </Tooltip>
        )}
      </div>
      <div>
        <Button
          children={t('cancel')}
          type="default"
          danger
          disabled={!hasChanges}
          onClick={cancelChanges}
        />
        {'  '}
        <Button
          children={hasChanges ? saveLabel : t('quit')}
          type="primary"
          danger={!hasChanges}
          onClick={SaveOrQuit}
        />
      </div>
    </div>
  );
};
