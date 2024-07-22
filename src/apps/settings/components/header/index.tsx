import { RouteExtraInfo } from '../navigation/routes';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { Button, Tooltip } from 'antd';
import { useTranslation } from 'react-i18next';
import { useDispatch } from 'react-redux';

import { SaveStore } from '../../modules/shared/store/infra';
import { useAppSelector } from '../../modules/shared/utils/infra';

import { RootActions } from '../../modules/shared/store/app/reducer';
import { RootSelectors } from '../../modules/shared/store/app/selectors';

import cs from './index.module.css';

export const Header = () => {
  let route = useAppSelector(RootSelectors.route);
  let hasChanges = useAppSelector(RootSelectors.toBeSaved);

  const { t } = useTranslation();

  const dispatch = useDispatch();

  const cancelChanges = () => {
    dispatch(RootActions.restoreToLastLoaded());
  };

  const SaveOrQuit = () => {
    if (hasChanges) {
      SaveStore();
    } else {
      getCurrentWebviewWindow().close();
    }
  };

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
          children={hasChanges ? t('save') : t('quit')}
          type="primary"
          danger={!hasChanges}
          onClick={SaveOrQuit}
        />
      </div>
    </div>
  );
};
