import { IconName } from '@icons';
import { ResourceId, ResourceKind, ResourceMetadata, Wallpaper } from '@seelen-ui/lib/types';
import { Icon } from '@shared/components/Icon';
import { ResourceText } from '@shared/components/ResourceText';
import { cx } from '@shared/styles';
import { convertFileSrc } from '@tauri-apps/api/core';
import { Button, Tooltip } from 'antd';
import { ComponentChildren } from 'preact';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { EnvConfig } from '../shared/config/infra';
import cs from './infra.module.css';

import { RootSelectors } from '../shared/store/app/selectors';

import { ExportResource } from '../shared/store/storeApi';

type AnyResource = {
  id: ResourceId;
  metadata: ResourceMetadata;
};

interface ResourceCardProps {
  kind: ResourceKind;
  resource: AnyResource;
  actions?: React.ReactNode;
}

export function ResourceCard({ resource, kind, actions }: ResourceCardProps) {
  const isDevToolsEnabled = useSelector(RootSelectors.devTools);

  const { t } = useTranslation();

  const [major = 0, minor = 0, patch = 0] = EnvConfig.version.split('.').map(Number);
  const [majorTarget = 0, minorTarget = 0, patchTarget = 0] =
    resource.metadata.appTargetVersion || [];

  const targetIsOlder =
    majorTarget < major ||
    (majorTarget === major && minorTarget < minor) ||
    (majorTarget === major && minorTarget === minor && patchTarget < patch);

  const targetIsNewer =
    majorTarget > major ||
    (majorTarget === major && minorTarget > minor) ||
    (majorTarget === major && minorTarget === minor && patchTarget > patch);

  const showWarning = targetIsOlder && !resource.metadata.bundled;
  const showDanger = targetIsNewer && !resource.metadata.bundled;

  return (
    <div
      className={cx(cs.card, {
        [cs.warn!]: showWarning,
        [cs.danger!]: showDanger,
      })}
    >
      <ResourcePortrait resource={resource} kind={kind}>
        {showWarning && (
          <Tooltip title={t('resources.outdated')}>
            <Icon iconName="IoWarning" className={cs.warning} />
          </Tooltip>
        )}
        {showDanger && (
          <Tooltip title={t('resources.app_outdated')}>
            <Icon iconName="IoWarning" className={cs.danger} />
          </Tooltip>
        )}
      </ResourcePortrait>
      <div className={cs.info}>
        <b>
          <ResourceText text={resource.metadata.displayName} />
        </b>
        <p>
          {resource.metadata.bundled || resource.id.startsWith('@user') ? (
            <span>{resource.id}</span>
          ) : (
            <a href={`https://seelen.io/resources/${resource.id.replace('@', '')}`} target="_blank">
              {resource.id}
            </a>
          )}
        </p>
      </div>
      <div className={cs.actions}>
        <div className={cs.actionsTop}>{actions}</div>
        {isDevToolsEnabled && kind !== 'Wallpaper' && (
          <Tooltip title={t('resources.export')} placement="left">
            <Button
              type="text"
              onClick={() => {
                ExportResource(resource);
              }}
            >
              <Icon iconName="BiExport" />
            </Button>
          </Tooltip>
        )}
      </div>
    </div>
  );
}

const icons: Record<ResourceKind, IconName> = {
  Theme: 'IoColorPaletteOutline',
  IconPack: 'LiaIconsSolid',
  Plugin: 'BsPlugin',
  Widget: 'BiSolidWidget',
  Wallpaper: 'LuWallpaper',
  SoundPack: 'PiWaveformBold',
};

interface ResourcePortraitProps {
  resource: AnyResource;
  kind: ResourceKind;
  children?: ComponentChildren;
}

export function ResourceIcon({ kind }: { kind: ResourceKind }) {
  return <Icon className={cs.kindIcon} iconName={icons[kind]} />;
}

function ResourcePortraitInner({ resource, kind }: ResourcePortraitProps) {
  if (resource.metadata.portrait) {
    return <img src={resource.metadata.portrait} />;
  }

  if (kind === 'Wallpaper') {
    const wallpaper = resource as Wallpaper;
    if (wallpaper.thumbnail_filename) {
      return (
        <img
          src={convertFileSrc(`${resource.metadata.path}\\${wallpaper.thumbnail_filename}`)}
          style={{ filter: 'blur(0.4px)' }}
          loading="lazy"
        />
      );
    }

    /* if (
        wallpaper.filename &&
        SUPPORTED_VIDEO_WALLPAPER_EXTENSIONS.includes(wallpaper.filename.split('.').pop()!)
      ) {
        return (
          <video
            src={convertFileSrc(`${resource.metadata.path}\\${wallpaper.filename}`)}
            controls={false}
            preload="metadata"
            style={{ filter: 'blur(0.4px)' }}
          />
        );
      } */
  }

  return <ResourceIcon kind={kind} />;
}

export function ResourcePortrait({ resource, kind, children }: ResourcePortraitProps) {
  return (
    <figure className={cs.portrait}>
      <ResourcePortraitInner resource={resource} kind={kind} />
      {children}
    </figure>
  );
}
