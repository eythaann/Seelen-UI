import { Reorder } from 'framer-motion';
import { useCallback, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';
import {
  HideMode,
  SeelenWegMode,
  SeelenWegSide,
  SeparatorWegItem,
  SwItemType,
  useWindowFocusChange,
} from 'seelen-core';

import { BackgroundByLayersV2 } from '../../components/BackgroundByLayers/infra';
import { FileOrFolder } from '../item/infra/File';
import { MediaSession } from '../item/infra/MediaSession';
import { StartMenu } from '../item/infra/StartMenu';
import { UserApplication } from '../item/infra/UserApplication';

import { RootActions, Selectors } from '../shared/store/app';

import { SwItem } from '../shared/store/domain';

import { cx } from '../../../shared/styles';
import { WithContextMenu } from '../../components/WithContextMenu';
import { savePinnedItems } from '../shared/store/storeApi';
import { getSeelenWegMenu } from './menu';

const Separator1: SeparatorWegItem = {
  id: '1',
  type: SwItemType.Separator,
};

const Separator2: SeparatorWegItem = {
  id: '2',
  type: SwItemType.Separator,
};

function shouldBeHidden(hideMode: HideMode, isActive: boolean, isOverlaped: boolean) {
  let shouldBeHidden = false;
  switch (hideMode) {
    case HideMode.Always:
      shouldBeHidden = !isActive;
      break;
    case HideMode.NeverWithoutPlaceholder:
    case HideMode.Never:
      shouldBeHidden = false;
      break;
    case HideMode.OnOverlap:
      shouldBeHidden = !isActive && isOverlaped;
  }
  return shouldBeHidden;
}

export function SeelenWeg() {
  const settings = useSelector(Selectors.settings);
  const isOverlaped = useSelector(Selectors.isOverlaped);

  const pinnedOnLeft = useSelector(Selectors.itemsOnLeft);
  const pinnedOnCenter = useSelector(Selectors.itemsOnCenter);
  const pinnedOnRight = useSelector(Selectors.itemsOnRight);

  const [isActive, setActive] = useState(false);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  useWindowFocusChange((focused) => {
    setActive(focused);
  });

  const getSeparatorComplementarySize = useCallback(
    (sideElements: number, centerElements: number) => {
      let size = '1px';

      if (settings.mode === SeelenWegMode.FullWidth) {
        size = `calc(50% - (${settings.size + settings.spaceBetweenItems}px * ${
          sideElements + centerElements / 2
        }) - ${settings.spaceBetweenItems}px)`;
      }

      if (settings.position === SeelenWegSide.Top || settings.position === SeelenWegSide.Bottom) {
        return {
          width: size,
        };
      }

      return {
        height: size,
      };
    },
    [settings],
  );

  const onReorderPinned = useCallback((apps: SwItem[]) => {
    let extractedPinned: SwItem[] = [];

    apps.forEach((app) => {
      if (app === Separator1) {
        dispatch(RootActions.setItemsOnLeft(extractedPinned));
        extractedPinned = [];
        return;
      }

      if (app === Separator2) {
        dispatch(RootActions.setItemsOnCenter(extractedPinned));
        extractedPinned = [];
        return;
      }

      if (app.type !== SwItemType.Separator) {
        extractedPinned.push(app);
      }
    });

    dispatch(RootActions.setItemsOnRight(extractedPinned));
    savePinnedItems();
  }, []);

  const isHorizontal =
    settings.position === SeelenWegSide.Top || settings.position === SeelenWegSide.Bottom;

  return (
    <WithContextMenu items={getSeelenWegMenu(t)}>
      <Reorder.Group
        as="div"
        values={[...pinnedOnLeft, Separator1, ...pinnedOnCenter, Separator2, ...pinnedOnRight]}
        onReorder={onReorderPinned}
        axis={isHorizontal ? 'x' : 'y'}
        className={cx('taskbar', settings.position.toLowerCase(), {
          horizontal: isHorizontal,
          vertical: !isHorizontal,
          'full-width': settings.mode === SeelenWegMode.FullWidth,
          hidden: shouldBeHidden(settings.hideMode, isActive, isOverlaped),
        })}
      >
        <BackgroundByLayersV2 prefix="taskbar" />
        {[
          ...pinnedOnLeft.map(ItemByType),
          <Reorder.Item
            as="div"
            key="separator1"
            value={Separator1}
            className={cx('weg-separator weg-separator-1', {
              visible: settings.visibleSeparators,
            })}
            drag={false}
            style={getSeparatorComplementarySize(pinnedOnLeft.length, pinnedOnCenter.length)}
          />,
          ...pinnedOnCenter.map(ItemByType),
          <Reorder.Item
            as="div"
            key="separator2"
            value={Separator2}
            className={cx('weg-separator weg-separator-2', {
              visible: settings.visibleSeparators,
            })}
            drag={false}
            style={getSeparatorComplementarySize(pinnedOnRight.length, pinnedOnCenter.length)}
          />,
          ...pinnedOnRight.map(ItemByType),
        ]}
      </Reorder.Group>
    </WithContextMenu>
  );
}

function ItemByType(item: SwItem) {
  if (item.type === SwItemType.Pinned && item.path) {
    if (
      item.execution_command.startsWith('shell:AppsFolder') ||
      item.execution_command.endsWith('.exe')
    ) {
      return <UserApplication key={item.execution_command} item={item} />;
    }
    return <FileOrFolder key={item.execution_command} item={item} />;
  }

  if (item.type === SwItemType.TemporalApp && item.path) {
    return <UserApplication key={item.execution_command} item={item} />;
  }

  if (item.type === SwItemType.Media) {
    return <MediaSession key="media-item" item={item} />;
  }

  if (item.type === SwItemType.Start) {
    return <StartMenu key="start-menu" item={item} />;
  }

  return null;
}
