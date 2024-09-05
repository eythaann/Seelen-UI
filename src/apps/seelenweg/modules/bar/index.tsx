import { SavedSeparatorItem } from '../../../shared/schemas/SeelenWegItems';
import { cx } from '../../../shared/styles';
import { WithContextMenu } from '../../components/WithContextMenu';
import { savePinnedItems } from '../shared/store/storeApi';
import { getSeelenWegMenu } from './menu';
import { Reorder } from 'framer-motion';
import { useCallback, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';
import { HideMode, SeelenWegMode, SeelenWegSide, useWindowFocusChange } from 'seelen-core';

import { BackgroundByLayersV2 } from '../../components/BackgroundByLayers/infra';
import { MediaSession } from '../item/infra/MediaSession';
import { StartMenu } from '../item/infra/StartMenu';
import { UserApplication } from '../item/infra/UserApplication';

import { RootActions, Selectors } from '../shared/store/app';

import { SpecialItemType, SwItem } from '../shared/store/domain';

import './index.css';

const Separator1: SavedSeparatorItem = {
  type: SpecialItemType.Separator,
};

const Separator2: SavedSeparatorItem = {
  type: SpecialItemType.Separator,
};

function shouldBeHidden(hideMode: HideMode, isActive: boolean, isOverlaped: boolean) {
  let shouldBeHidden = false;
  switch (hideMode) {
    case HideMode.Always:
      shouldBeHidden = !isActive;
      break;
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

  const onReorderPinned = useCallback((apps: (SavedSeparatorItem | SwItem)[]) => {
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

      if (app.type !== SpecialItemType.Separator) {
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
  if (item.type === SpecialItemType.PinnedApp || item.type === SpecialItemType.TemporalApp) {
    return <UserApplication key={item.exe || item.opens[0] || item.title} item={item} />;
  }

  if (item.type === SpecialItemType.Media) {
    return <MediaSession key={'media-item'} item={item} />;
  }

  if (item.type === SpecialItemType.Start) {
    return <StartMenu key={'start-menu'} item={item} />;
  }

  return null;
}
