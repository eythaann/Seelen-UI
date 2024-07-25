import { StartMenuItem } from '../../../../shared/schemas/SeelenWegItems';
import { WithContextMenu } from '../../../components/WithContextMenu';
import { DraggableItem } from './DraggableItem';
import { StartModuleMenu } from './Menu';
import { invoke } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';
import { memo, useEffect, useRef } from 'react';
import { useSelector } from 'react-redux';

import { BackgroundByLayers } from '../../../components/BackgrounByLayers/infra';

import { Selectors } from '../../shared/store/app';

import { RootState } from '../../shared/store/domain';

interface Props {
  item: StartMenuItem;
}

const startMenuExes = ['SearchHost.exe', 'StartMenuExperienceHost.exe'];

export const StartMenu = memo(({ item }: Props) => {
  const startMenuOpenRef = useRef(false);

  const themeLayers = useSelector(Selectors.themeLayers);
  const size = useSelector(Selectors.settings.size);

  const isStartMenuOpen = useSelector((state: RootState) =>
    startMenuExes.includes(Selectors.focusedExecutable(state)),
  );

  useEffect(() => {
    if (!isStartMenuOpen) {
      setTimeout(() => {
        startMenuOpenRef.current = isStartMenuOpen;
      }, 100);
    } else {
      startMenuOpenRef.current = isStartMenuOpen;
    }
  }, [isStartMenuOpen]);

  return (
    <DraggableItem item={item}>
      <WithContextMenu items={StartModuleMenu}>
        <motion.div
          className="weg-item"
          initial={{ scale: 0 }}
          animate={{ scale: 1 }}
          style={{ height: size, aspectRatio: '1/1' }}
          onClick={() => {
            if (!startMenuOpenRef.current) {
              invoke('send_keys', { keys: '{win}' });
            }
          }}
          onContextMenu={(e) => e.stopPropagation()}
        >
          <BackgroundByLayers prefix="item" layers={themeLayers.weg.items.bg} />
          <div className="weg-item-icon">
            <div className="weg-item-icon-start" />
          </div>
        </motion.div>
      </WithContextMenu>
    </DraggableItem>
  );
});
