import { useSignal } from "@preact/signals";
import { ResourceKind, type WallpaperId } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { VerticalSortableSelect } from "src/ui/react/settings/components/SortableSelector/index.tsx";
import { Wallpaper } from "libs/ui/react/components/Wallpaper/index.tsx";
import { Button, Modal } from "antd";

import { getWallpaperCollections, updateWallpaperCollection } from "./application.ts";
import { wallpapers } from "../../state/resources.ts";

import { ResourcePortrait } from "../resources/ResourceCard.tsx";
import cs from "./index.module.css";

interface Props {
  collectionId: string;
}

export function WallpaperList({ collectionId }: Props) {
  const $toPreview = useSignal<WallpaperId | null>(null);

  const wallpaperCollections = getWallpaperCollections();
  const collection = wallpaperCollections.find((c) => c.id === collectionId);

  function onChangeEnabled(wallpaperIds: WallpaperId[]) {
    if (!collection) {
      return;
    }

    updateWallpaperCollection({
      ...collection,
      wallpapers: wallpaperIds,
    });
  }

  if (!collection) {
    return null;
  }

  const previewing = $toPreview.value ? wallpapers.value.find((w) => w.id === $toPreview.value) : null;

  return (
    <div style={{ height: "60vh" }}>
      <VerticalSortableSelect
        options={wallpapers.value.map((w) => ({
          value: w.id,
          label: (
            <div className={cs.entryLabel}>
              <ResourcePortrait resource={w} kind={ResourceKind.Wallpaper} />
              <ResourceText className={cs.entryName} text={w.metadata.displayName} />
              <Button type="text" size="small" onClick={() => ($toPreview.value = w.id)}>
                <Icon iconName="FaEye" />
              </Button>
            </div>
          ),
        }))}
        enabled={collection.wallpapers}
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
          {previewing && <Wallpaper definition={previewing} muted />}
        </div>
      </Modal>
    </div>
  );
}
