import { SeelenWegMode, SeelenWegSide, WegItemType } from '@seelen-ui/lib';
import { Reorder } from 'framer-motion';
import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from '../../components/BackgroundByLayers/infra';
import { FileOrFolder } from '../item/infra/File';
import { MediaSession } from '../item/infra/MediaSession';
import { StartMenu } from '../item/infra/StartMenu';
import { UserApplication } from '../item/infra/UserApplication';

import { RootActions, Selectors } from '../shared/store/app';

import { SeparatorWegItem, SwItem } from '../shared/store/domain';

import { cx } from '../../../shared/styles';
import { WithContextMenu } from '../../components/WithContextMenu';
import { $dock_should_be_hidden, $settings } from '../shared/state/mod';
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

export function SeelenWeg() {
  const isReorderDisabled = useSelector(Selectors.reorderDisabled);
  const pinnedOnLeft = useSelector(Selectors.itemsOnLeft);
  const pinnedOnCenter = useSelector(Selectors.itemsOnCenter);
  const pinnedOnRight = useSelector(Selectors.itemsOnRight);

  const dispatch = useDispatch();
  const { t } = useTranslation();

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

  const settings = $settings.value;
  const isHorizontal =
    settings.position === SeelenWegSide.Top || settings.position === SeelenWegSide.Bottom;

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
          hidden: $dock_should_be_hidden.value,
        })}
      >
        <BackgroundByLayersV2 prefix="taskbar" />
        <div className="weg-items-container">
          <div className="weg-items">
            {pinnedOnLeft.length + pinnedOnCenter.length + pinnedOnRight.length == 0 && (
              <span className="weg-empty-state-label">{t('weg.empty')}</span>
            )}
            {isTemporalOnlyWegBar
              ? [
                ...pinnedOnLeft.map(ItemByType),
                ...pinnedOnCenter.map(ItemByType),
                ...pinnedOnRight.map(ItemByType),
              ]
              : [
                ...pinnedOnLeft.map(ItemByType),
                <Reorder.Item
                  as="div"
                  key="separator1"
                  value={Separator1}
                  className={cx('weg-separator weg-separator-1', {
                    visible: settings.visibleSeparators,
                  })}
                  drag={false}
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
                />,
                ...pinnedOnRight.map(ItemByType),
              ]}
          </div>
        </div>
      </Reorder.Group>
    </WithContextMenu>
  );
}

function ItemByType(item: SwItem) {
  if (item.type === WegItemType.Pinned) {
    if (item.subtype === 'App') {
      return <UserApplication key={item.id} item={item} />;
    }
    return <FileOrFolder key={item.id} item={item} />;
  }

  if (item.type === WegItemType.Temporal) {
    return <UserApplication key={item.id} item={item} />;
  }

  if (item.type === WegItemType.Media) {
    return <MediaSession key={item.id} item={item} />;
  }

  if (item.type === WegItemType.StartMenu) {
    return <StartMenu key={item.id} item={item} />;
  }

  return null;
}
