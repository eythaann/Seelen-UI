import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';
import { memo, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';
import { PinnedWegItem, SeelenCommand } from 'seelen-core';

import { BackgroundByLayersV2 } from '../../../components/BackgroundByLayers/infra';
import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { Selectors } from '../../shared/store/app';
import InlineSVG from 'src/apps/seelenweg/components/InlineSvg';

import { WithContextMenu } from '../../../components/WithContextMenu';
import { DraggableItem } from './DraggableItem';
import { getMenuForItem } from './Menu';

interface Props {
  item: PinnedWegItem;
}

export const FileOrFolder = memo(({ item }: Props) => {
  const size = useSelector(Selectors.settings.size);

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
        <motion.div
          className="weg-item"
          initial={{ scale: 0 }}
          animate={{ scale: 1 }}
          style={{ height: size, aspectRatio: '1/1' }}
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
        </motion.div>
      </WithContextMenu>
    </DraggableItem>
  );
});
