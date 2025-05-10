import { IconName } from '@icons';
import { ResourceId, ResourceKind, ResourceMetadata } from '@seelen-ui/lib/types';
import { Icon } from '@shared/components/Icon';
import { ResourceText } from '@shared/components/ResourceText';

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
  return (
    <div className={cs.card}>
      <figure className={cs.portrait}>
        {resource.metadata.portrait ? (
          <img src={resource.metadata.portrait} />
        ) : (
          <ResourceKindIcon kind={kind} />
        )}
      </figure>
      <div className={cs.info}>
        <b>
          <ResourceText text={resource.metadata.displayName} />
        </b>
        <p>
          {resource.id.startsWith('@default') ? (
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
