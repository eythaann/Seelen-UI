import { useSignal } from '@preact/signals';
import { WallpaperConfiguration } from '@seelen-ui/lib';
import { WallpaperId } from '@seelen-ui/lib/types';
import { Icon } from '@shared/components/Icon';
import { ResourceText } from '@shared/components/ResourceText';
import { VerticalSortableSelect } from '@shared/components/SortableSelector';
import { Wallpaper } from '@shared/components/Wallpaper';
import { Button, Modal } from 'antd';
import { useDispatch, useSelector } from 'react-redux';

import { newSelectors, RootActions } from '../shared/store/app/reducer';

import { ResourcePortrait } from '../resources/ResourceCard';
import cs from './index.module.css';

const defaultWallpaperConfig = await WallpaperConfiguration.default();

export function WallpaperList() {
  const $toPreview = useSignal<WallpaperId | null>(null);

  const enabled = useSelector(newSelectors.wall.backgroundsV2);
  const wallpapers = useSelector(newSelectors.wallpapers);

  const d = useDispatch();

  function onChangeEnabled(backgroundsV2: WallpaperId[]) {
    d(RootActions.patchWall({ backgroundsV2 }));
  }

  const previewing = $toPreview.value ? wallpapers.find((w) => w.id === $toPreview.value) : null;
  return (
    <>
      <VerticalSortableSelect
        options={wallpapers.map((w) => ({
          value: w.id,
          label: (
            <div className={cs.entryLabel}>
              <ResourcePortrait resource={w} kind="Wallpaper" />
              <ResourceText className={cs.entryName} text={w.metadata.displayName} />
              <Button type="text" size="small" onClick={() => ($toPreview.value = w.id)}>
                <Icon iconName="FaEye" />
              </Button>
            </div>
          ),
        }))}
        enabled={enabled}
        onChange={onChangeEnabled}
      />
      <Modal
        open={!!previewing}
        title={<ResourceText text={previewing?.metadata.displayName} />}
        onCancel={() => ($toPreview.value = null)}
        footer={null}
        centered
      >
        <div className={cs.preview}>
          {previewing && <Wallpaper definition={previewing} config={defaultWallpaperConfig} />}
        </div>
      </Modal>
    </>
  );
}
