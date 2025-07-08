import { SeelenCommand } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';
import { memo } from 'react';
import { useTranslation } from 'react-i18next';

import { BackgroundByLayersV2 } from '../../../components/BackgroundByLayers/infra';

import { FileIcon, SpecificIcon } from 'src/apps/shared/components/Icon';

import { PinnedWegItem } from '../../shared/store/domain';

import { WithContextMenu } from '../../../components/WithContextMenu';
import { getMenuForItem } from './Menu';

interface Props {
  item: PinnedWegItem;
}

export const FileOrFolder = memo(({ item }: Props) => {
  const { t } = useTranslation();

  return (
    <WithContextMenu items={getMenuForItem(t, item)}>
      <div
        className="weg-item"
        onClick={() => {
          invoke(SeelenCommand.OpenFile, { path: item.path });
        }}
        onContextMenu={(e) => e.stopPropagation()}
      >
        <BackgroundByLayersV2 prefix="item" />
        {item.subtype === 'Folder' ? (
          <SpecificIcon
            className="weg-item-icon weg-item-folder-icon"
            name="@seelen/weg::folder"
          />
        ) : (
          <FileIcon className="weg-item-icon" path={item.path} umid={item.umid} />
        )}
      </div>
    </WithContextMenu>
  );
});
