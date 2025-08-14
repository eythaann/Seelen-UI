import { useSignal } from '@preact/signals';
import { SeelenWallWidgetId } from '@seelen-ui/lib';
import { WallpaperId } from '@seelen-ui/lib/types';
import { Icon } from '@shared/components/Icon';
import { ResourceText } from '@shared/components/ResourceText';
import { VerticalSortableSelect } from '@shared/components/SortableSelector';
import { Wallpaper } from '@shared/components/Wallpaper';
import { Button, Modal, Switch } from 'antd';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { newSelectors, RootActions } from '../shared/store/app/reducer';

import { SettingsGroup, SettingsOption } from '../../components/SettingsBox';
import { ResourcePortrait } from '../resources/ResourceCard';
import cs from './index.module.css';

interface Props {
  monitorId?: string;
}

export function WallpaperList({ monitorId }: Props) {
  const $toPreview = useSignal<WallpaperId | null>(null);

  const baseEnabled = useSelector(newSelectors.wall.backgroundsV2);

  const configByMonitor = useSelector(newSelectors.monitorsV3);
  const monitorPatch = monitorId ? configByMonitor[monitorId] : null;

  const wallpapers = useSelector(newSelectors.wallpapers);

  const d = useDispatch();
  const { t } = useTranslation();

  function onChangeEnabled(backgroundsV2: WallpaperId[]) {
    if (!monitorId) {
      d(RootActions.patchWall({ backgroundsV2 }));
      return;
    }

    d(
      RootActions.patchWidgetMonitorConfig({
        monitorId,
        widgetId: SeelenWallWidgetId,
        config: { backgroundsV2 },
      }),
    );
  }

  function setInherited(value: boolean) {
    if (!monitorId) {
      return;
    }

    d(
      RootActions.patchWidgetMonitorConfig({
        monitorId,
        widgetId: SeelenWallWidgetId,
        config: { backgroundsV2: value ? null : [] },
      }),
    );
  }

  const enabledOnMonitor = monitorPatch?.byWidget[SeelenWallWidgetId]
    ?.backgroundsV2 as WallpaperId[];
  const isInherited = !!monitorId && !enabledOnMonitor;
  const enabled = enabledOnMonitor ?? baseEnabled;

  const previewing = $toPreview.value ? wallpapers.find((w) => w.id === $toPreview.value) : null;
  return (
    <SettingsGroup>
      {monitorId && (
        <SettingsOption
          label={t('inherit')}
          action={<Switch value={isInherited} onChange={setInherited} />}
        />
      )}
      <VerticalSortableSelect
        disabled={isInherited}
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
          {previewing && <Wallpaper definition={previewing} />}
        </div>
      </Modal>
    </SettingsGroup>
  );
}
