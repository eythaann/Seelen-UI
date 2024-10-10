import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { memo, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { PinnedWegItem, SeelenCommand } from 'seelen-core';

import { BackgroundByLayersV2 } from '../../../components/BackgroundByLayers/infra';
import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import InlineSVG from 'src/apps/seelenweg/components/InlineSvg';

import { WithContextMenu } from '../../../components/WithContextMenu';
import { DraggableItem } from './DraggableItem';
import { getMenuForItem } from './Menu';

interface Props {
  item: PinnedWegItem;
}

export const FileOrFolder = memo(({ item }: Props) => {
  const [iconSrc, setIconSrc] = useState<string>(
    item.is_dir ? convertFileSrc(LAZY_CONSTANTS.FOLDER_ICON_PATH) : '',
  );

  const { t } = useTranslation();

  useEffect(() => {
    if (!item.is_dir) {
      invoke<string | null>(SeelenCommand.GetIcon, { path: item.path }).then((icon) => {
        setIconSrc(convertFileSrc(icon || LAZY_CONSTANTS.MISSING_ICON_PATH));
      });
    }
  }, [item]);

  return (
    <DraggableItem item={item}>
      <WithContextMenu items={getMenuForItem(t, item)}>
        <div
          className="weg-item"
          onClick={() => {
            invoke(SeelenCommand.OpenFile, { path: item.path });
          }}
          onContextMenu={(e) => e.stopPropagation()}
        >
          <BackgroundByLayersV2 prefix="item" />
          {iconSrc.endsWith('.svg') ? (
            <InlineSVG className="weg-item-icon" src={iconSrc} />
          ) : (
            <img className="weg-item-icon" src={iconSrc} draggable={false} />
          )}
        </div>
      </WithContextMenu>
    </DraggableItem>
  );
});
