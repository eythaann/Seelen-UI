import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { relaunch } from '@tauri-apps/plugin-process';
import { Button } from 'antd';
import React from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch } from 'react-redux';
import { useLocation } from 'react-router';

import { SaveStore } from '../../modules/shared/store/infra';
import { useAppSelector } from '../../modules/shared/utils/infra';

import { RootActions } from '../../modules/shared/store/app/reducer';
import { RootSelectors } from '../../modules/shared/store/app/selectors';

import { RouteExtraInfo } from './ExtraInfo';
import { UpdateButton } from './UpdateButton';
import cs from './index.module.css';

export const Header = () => {
  let location = useLocation();
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
  const ExtraInfo = RouteExtraInfo[location.pathname];

  const parts = location.pathname === '/' ? ['home'] : location.pathname.split('/').filter(Boolean);

  return (
    <div className={cs.Header} data-tauri-drag-region>
      <div className={cs.title}>
        {parts.map((part, idx) => (
          <React.Fragment key={part}>
            <span>{t(`header.labels.${part}`)}</span>
            {++idx < parts.length ? '>' : ''}
          </React.Fragment>
        ))}
        {ExtraInfo && <ExtraInfo />}
      </div>
      <div className={cs.actions}>
        <UpdateButton />
        <Button
          style={{ minWidth: 60 }}
          children={t('cancel')}
          type="default"
          danger
          disabled={!hasChanges}
          onClick={cancelChanges}
        />
        <Button
          style={{ minWidth: 60 }}
          children={hasChanges ? saveLabel : t('close')}
          type="primary"
          danger={!hasChanges}
          onClick={SaveOrQuit}
        />
      </div>
    </div>
  );
};
