import { process } from '@seelen-ui/lib/tauri';
import { ResourceText } from '@shared/components/ResourceText';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { Button } from 'antd';
import React from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';
import { useLocation } from 'react-router';

import { SaveStore } from '../../modules/shared/store/infra';
import { useAppSelector } from '../../modules/shared/utils/infra';

import { RootActions } from '../../modules/shared/store/app/reducer';
import { RootSelectors } from '../../modules/shared/store/app/selectors';

import { RouteExtraInfo } from './ExtraInfo';
import { UpdateButton } from './UpdateButton';
import cs from './index.module.css';

export const Header = () => {
  const widgets = useSelector(RootSelectors.widgets);
  const themes = useSelector(RootSelectors.availableThemes);

  const hasChanges = useAppSelector(RootSelectors.toBeSaved);
  const shouldRestart = useAppSelector(RootSelectors.toBeRestarted);

  const location = useLocation();
  const dispatch = useDispatch();
  const { t } = useTranslation();

  const cancelChanges = () => {
    dispatch(RootActions.restoreToLastLoaded());
  };

  const SaveOrQuit = async () => {
    if (hasChanges) {
      await SaveStore();
      if (shouldRestart) {
        await process.relaunch();
      }
    } else {
      await getCurrentWebviewWindow().close();
    }
  };

  const saveBtnLabel = shouldRestart ? t('save_and_restart') : t('save');

  let label: React.ReactNode = <span>null!?</span>;
  let parts = location.pathname === '/' ? ['home'] : location.pathname.split('/').filter(Boolean);

  if (parts[0] === 'widget') {
    const [_, username, resourceName] = parts;
    const widgetId = `@${username}/${resourceName}`;
    const widget = widgets.find((w) => w.id === widgetId);
    label = widget ? <ResourceText text={widget.metadata.displayName} /> : <span>{widgetId}</span>;
  } else if (parts[0] === 'theme') {
    const [_, username, resourceName] = parts;
    const themeId = `@${username}/${resourceName}`;
    const theme = themes.find((t) => t.id === themeId);
    label = theme ? <ResourceText text={theme.metadata.displayName} /> : <span>{themeId}</span>;
  } else {
    if (parts[0] === 'wallpaper') {
      parts = ['resources', 'wallpaper', 'config'];
    }

    label = parts.map((part, idx) => (
      <React.Fragment key={part}>
        <span className={cs.part}>{t(`header.labels.${part}`)}</span>
        {++idx < parts.length ? '>' : ''}
      </React.Fragment>
    ));
  }

  const ExtraInfo = RouteExtraInfo[location.pathname];

  return (
    <div className={cs.Header} data-tauri-drag-region>
      <div className={cs.title}>
        {label}
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
          children={hasChanges ? saveBtnLabel : t('close')}
          type="primary"
          danger={!hasChanges}
          onClick={SaveOrQuit}
        />
      </div>
    </div>
  );
};
