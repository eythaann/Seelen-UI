import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { relaunch } from '@tauri-apps/plugin-process';
import { check as getUpdate, Update } from '@tauri-apps/plugin-updater';
import { Badge, Button, Tooltip } from 'antd';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch } from 'react-redux';
import { SeelenCommand } from 'seelen-core';

import { SaveStore } from '../../modules/shared/store/infra';
import { useAppSelector } from '../../modules/shared/utils/infra';

import { RootActions } from '../../modules/shared/store/app/reducer';
import { RootSelectors } from '../../modules/shared/store/app/selectors';
import { wasInstalledUsingMSIX } from 'src/apps/shared';
import { Icon } from 'src/apps/shared/components/Icon';

import { RouteExtraInfo } from '../navigation/routes';
import cs from './index.module.css';

export const Header = () => {
  let [update, setUpdate] = useState<Update | null>(null);

  let route = useAppSelector(RootSelectors.route);
  let hasChanges = useAppSelector(RootSelectors.toBeSaved);
  let shouldRestart = useAppSelector(RootSelectors.toBeRestarted);

  const { t } = useTranslation();

  const dispatch = useDispatch();

  useEffect(() => {
    async function checkUpdate() {
      let isDev = await invoke<boolean>(SeelenCommand.IsDevMode);
      let isMsix = await wasInstalledUsingMSIX();
      if (!isDev && !isMsix) {
        let update = await getUpdate();
        if (update) {
          await update.download();
          setUpdate(update);
        }
      }
    }
    checkUpdate();
  }, []);

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
      <div className={cs.title}>
        {t(`header.labels.${route}`)}
        {RouteExtraInfo[route] && (
          <Tooltip title={RouteExtraInfo[route]}>
            <span className={cs.info}>🛈</span>
          </Tooltip>
        )}
      </div>
      <div className={cs.actions}>
        {update && (
          <Tooltip title={t('update.available')}>
            <Button type="text" onClick={() => update?.install()}>
              <Badge dot>
                <Icon iconName="TbDownload" />
              </Badge>
            </Button>
          </Tooltip>
        )}
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
