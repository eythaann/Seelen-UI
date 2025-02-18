import { HideMode, SeelenWegMode, SeelenWegSide, WegItemType } from '@seelen-ui/lib';
import { Reorder } from 'framer-motion';
import { useCallback, useLayoutEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../components/BackgroundByLayers/infra';
import { FileOrFolder } from '../item/infra/File';
import { MediaSession } from '../item/infra/MediaSession';
import { StartMenu } from '../item/infra/StartMenu';
import { UserApplication } from '../item/infra/UserApplication';

import { RootActions, Selectors } from '../shared/store/app';
import { useWindowFocusChange } from 'src/apps/shared/hooks';

import { SeparatorWegItem, SwItem } from '../shared/store/domain';

import { cx } from '../../../shared/styles';
import { WithContextMenu } from '../../components/WithContextMenu';
import { savePinnedItems } from '../shared/store/storeApi';
import { getSeelenWegMenu } from './menu';

const Separator1: SeparatorWegItem = {
  id: '1',
  type: WegItemType.Separator,
};

const Separator2: SeparatorWegItem = {
  id: '2',
  type: WegItemType.Separator,
};

function shouldBeHidden(
  hideMode: HideMode,
  isActive: boolean,
  isOverlaped: boolean,
  associatedViewCounter: number,
) {
  let shouldBeHidden = false;
  switch (hideMode) {
    case HideMode.Always:
      shouldBeHidden = !isActive && associatedViewCounter == 0;
      break;
    case HideMode.Never:
      shouldBeHidden = false;
      break;
    case HideMode.OnOverlap:
      shouldBeHidden = !isActive && isOverlaped && associatedViewCounter == 0;
  }
  return shouldBeHidden;
}

function calculateAssociatedViewCounter(currentValue: number, currentChange: boolean): number {
  const newValue = currentValue + (currentChange ? 1 : -1);
  return newValue >= 0 ? newValue : currentValue;
}

export function SeelenWeg() {
  const [isActive, setActive] = useState(false);
  const [delayed, setDelayed] = useState(false);
  // Counts every associated window in the bar and will act as a reverse mutex for the hide functionality
  const [associatedViewCounter, setAssociatedViewCounter] = useState(0);

  const settings = useSelector(Selectors.settings);
  const isOverlaped = useSelector(Selectors.isOverlaped);

  let isReorderDisabled: boolean = useSelector(Selectors.reorderDisabled);
  const pinnedOnLeft = useSelector(Selectors.itemsOnLeft);
  const pinnedOnCenter = useSelector(Selectors.itemsOnCenter);
  const pinnedOnRight = useSelector(Selectors.itemsOnRight);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  useWindowFocusChange((focused) => {
    if (focused) setAssociatedViewCounter(0);
    setActive(focused);
  });

  useLayoutEffect(() => {
    switch (settings.hideMode) {
      case HideMode.Always:
        setDelayed(true);
        break;
      case HideMode.Never:
        setDelayed(false);
        break;
      case HideMode.OnOverlap:
        if (!isOverlaped) {
          setDelayed(false);
          break;
        }
        setTimeout(() => {
          setDelayed(true);
        }, 300);
        break;
    }
  }, [isOverlaped, settings]);

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

      if (app.type !== WegItemType.Separator) {
        extractedPinned.push(app);
      }
    });

    if (isTemporalOnlyWegBar) {
      dispatch(RootActions.setItemsOnLeft([]));
      dispatch(RootActions.setItemsOnCenter([]));
    }

    dispatch(RootActions.setItemsOnRight(extractedPinned));
    savePinnedItems();
  }, []);

  const isTemporalOnlyWegBar = !(
    pinnedOnLeft.some((item) => 'pinDisabled' in item && !item.pinDisabled) ||
    pinnedOnCenter.some((item) => 'pinDisabled' in item && !item.pinDisabled) ||
    pinnedOnRight.some((item) => 'pinDisabled' in item && !item.pinDisabled)
  );

  const isHorizontal =
    settings.position === SeelenWegSide.Top || settings.position === SeelenWegSide.Bottom;

  const shit = useCallback((isOpen: boolean) => {
    setAssociatedViewCounter((current) => calculateAssociatedViewCounter(current, isOpen));
  }, []);

  const projectSwItem = (item: SwItem) => ItemByType(item, !isReorderDisabled, shit);

  return (
    <WithContextMenu items={getSeelenWegMenu(t, isTemporalOnlyWegBar, isReorderDisabled)}>
      <Reorder.Group
        as="div"
        values={
          isTemporalOnlyWegBar
            ? [...pinnedOnLeft, ...pinnedOnCenter, ...pinnedOnRight]
            : [...pinnedOnLeft, Separator1, ...pinnedOnCenter, Separator2, ...pinnedOnRight]
        }
        onReorder={onReorderPinned}
        axis={isHorizontal ? 'x' : 'y'}
        className={cx('taskbar', settings.position.toLowerCase(), {
          horizontal: isHorizontal,
          vertical: !isHorizontal,
          'temporal-only': isTemporalOnlyWegBar,
          'full-width': settings.mode === SeelenWegMode.FullWidth,
          hidden: shouldBeHidden(settings.hideMode, isActive, isOverlaped, associatedViewCounter),
          delayed,
        })}
      >
        <BackgroundByLayersV2 prefix="taskbar" />
        {pinnedOnLeft.length + pinnedOnCenter.length + pinnedOnRight.length == 0 && (
          <span className="weg-empty-state-label">{t('weg.empty')}</span>
        )}
        {isTemporalOnlyWegBar
          ? [
            ...pinnedOnLeft.map(projectSwItem),
            ...pinnedOnCenter.map(projectSwItem),
            ...pinnedOnRight.map(projectSwItem),
          ]
          : [
            ...pinnedOnLeft.map(projectSwItem),
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
            ...pinnedOnCenter.map(projectSwItem),
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
            ...pinnedOnRight.map(projectSwItem),
          ]}
      </Reorder.Group>
    </WithContextMenu>
  );
}

function ItemByType(item: SwItem, drag: boolean, callback: (isOpen: boolean) => void) {
  if (item.type === WegItemType.Pinned) {
    if (item.subtype === 'App') {
      return <UserApplication key={item.id} item={item} drag={drag} onAssociatedViewOpenChanged={callback} />;
    }
    return <FileOrFolder key={item.id} item={item} />;
  }

  if (item.type === WegItemType.Temporal) {
    return <UserApplication key={item.id} item={item} drag={drag} onAssociatedViewOpenChanged={callback} />;
  }

  if (item.type === WegItemType.Media) {
    return <MediaSession key={item.id} item={item} drag={drag} />;
  }

  if (item.type === WegItemType.StartMenu) {
    return <StartMenu key={item.id} item={item} drag={drag} />;
  }

  return null;
}
