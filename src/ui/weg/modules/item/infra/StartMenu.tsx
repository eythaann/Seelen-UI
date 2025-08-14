import { SeelenCommand } from '@seelen-ui/lib';
import { SpecificIcon } from '@shared/components/Icon';
import { invoke } from '@tauri-apps/api/core';
import { memo } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '@shared/components/BackgroundByLayers/infra';

import { Selectors } from '../../shared/store/app';

import { StartMenuWegItem } from '../../shared/store/domain';

import { WithContextMenu } from '../../../components/WithContextMenu';
import { getMenuForItem } from './Menu';

interface Props {
  item: StartMenuWegItem;
}

const startMenuExes = ['SearchHost.exe', 'StartMenuExperienceHost.exe'];

export const StartMenu = memo(({ item }: Props) => {
  const focused = useSelector(Selectors.focusedApp);

  const { t } = useTranslation();

  const isStartMenuOpen = startMenuExes.some((program) => (focused?.exe || '').endsWith(program));

  return (
    <WithContextMenu items={getMenuForItem(t, item)}>
      <div
        className="weg-item weg-item-start"
        onClick={() => {
          if (!isStartMenuOpen) {
            invoke(SeelenCommand.SendKeys, { keys: '{win}' });
          }
        }}
        onContextMenu={(e) => e.stopPropagation()}
      >
        <BackgroundByLayersV2 />
        <SpecificIcon
          className="weg-item-icon weg-item-start-icon"
          name={'@seelen/weg::start-menu'}
        />
      </div>
    </WithContextMenu>
  );
});
