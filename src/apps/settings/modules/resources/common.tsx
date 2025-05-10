import { IconName } from '@icons';
import { ResourceId, ResourceKind, ResourceMetadata } from '@seelen-ui/lib/types';
import { Icon } from '@shared/components/Icon';
import { ResourceText } from '@shared/components/ResourceText';
import { cx } from '@shared/styles';
import { Tooltip } from 'antd';
import { useTranslation } from 'react-i18next';

import { EnvConfig } from '../shared/config/infra';
import cs from './infra.module.css';

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
      <figure className={cs.portrait}>
        {resource.metadata.portrait ? (
          <img src={resource.metadata.portrait} />
        ) : (
          <ResourceKindIcon kind={kind} />
        )}
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
      </figure>
      <div className={cs.info}>
        <b>
          <ResourceText text={resource.metadata.displayName} />
        </b>
        <p>
          {resource.metadata.bundled ? (
            <span>{resource.id}</span>
          ) : (
            <a href={`https://seelen.io/resources/${resource.id.replace('@', '')}`} target="_blank">
              {resource.id}
            </a>
          )}
        </p>
      </div>
      <div className={cs.actions}>{actions}</div>
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

export function ResourceKindIcon({ kind }: { kind: ResourceKind }) {
  return <Icon className={cs.kindIcon} iconName={icons[kind]} />;
}
