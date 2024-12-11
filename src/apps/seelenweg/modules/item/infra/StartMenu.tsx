import { SeelenCommand } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';
import { memo } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../../components/BackgroundByLayers/infra';

import { Selectors } from '../../shared/store/app';

import { StartMenuWegItem } from '../../shared/store/domain';

import { WithContextMenu } from '../../../components/WithContextMenu';
import { DraggableItem } from './DraggableItem';
import { getMenuForItem } from './Menu';

interface Props {
  item: StartMenuWegItem;
  drag: boolean | 'x' | 'y' | undefined;
}

const startMenuExes = ['SearchHost.exe', 'StartMenuExperienceHost.exe'];

export const StartMenu = memo(({ item, drag }: Props) => {
  const focused = useSelector(Selectors.focusedApp);

  const { t } = useTranslation();

  const isStartMenuOpen = startMenuExes.some((program) => (focused?.exe || '').endsWith(program));

  return (
    <DraggableItem drag={drag} item={item}>
      <WithContextMenu items={getMenuForItem(t, item)}>
        <div
          className="weg-item"
          onClick={() => {
            if (!isStartMenuOpen) {
              invoke(SeelenCommand.SendKeys, { keys: '{win}' });
            }
          }}
          onContextMenu={(e) => e.stopPropagation()}
        >
          <BackgroundByLayersV2 prefix="item" />
          <div className="weg-item-icon">
            <div className="weg-item-icon-start" />
          </div>
        </div>
      </WithContextMenu>
    </DraggableItem>
  );
});
