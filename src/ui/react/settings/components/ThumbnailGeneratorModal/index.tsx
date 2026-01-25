import { Button, Flex, List, Modal, Progress } from "antd";
import { useEffect } from "react";
import { useTranslation } from "react-i18next";

import { $corruptedWallpapers } from "../../modules/shared/signals.ts";
import { generateThumbnails, getVideosWithoutThumbnail, type ThumbGenerationProgress } from "./thumbnailGenerator.ts";
import { useSignal } from "@preact/signals";
import { wallpapers } from "../../state/resources.ts";

export function ThumbnailGeneratorModal() {
  const { t } = useTranslation();

  const open = useSignal(false);
  const progress = useSignal<ThumbGenerationProgress | null>(null);

  useEffect(() => {
    // Do nothing if already processing
    if (open.value) {
      return;
    }

    const videosWithoutThumbnail = getVideosWithoutThumbnail(wallpapers.value);

    if (videosWithoutThumbnail.length > 0) {
      open.value = true;
      progress.value = null;

      generateThumbnails(videosWithoutThumbnail, (newProgress) => {
        progress.value = newProgress;
      });
    }
  }, [wallpapers, progress.value, open.value]);

  const percent = progress.value && progress.value.total > 0
    ? Math.round((progress.value.current / progress.value.total) * 100)
    : 0;

  // Get corrupted wallpaper names
  const corruptedWallpapers = wallpapers.value.filter((w) => $corruptedWallpapers.value.has(w.id));
  const isFinished = percent === 100;

  return (
    <Modal
      open={open.value}
      title={isFinished ? t("wall.thumbnail_generation_complete") : t("wall.generating_thumbnails")}
      footer={null}
      closable={isFinished}
      onCancel={() => {
        open.value = false;
      }}
      centered
    >
      {!isFinished
        ? (
          <Flex vertical align="center" justify="center" gap="16px">
            <p>
              {t("wall.processing_video")} {`${(progress.value?.current || 0) + 1} / ${progress.value?.total || 0}`}
            </p>
            <p>{progress.value?.currentVideoName}</p>
            <Progress percent={percent} status="active" />
          </Flex>
        )
        : (
          <Flex vertical gap="16px">
            <p>{t("wall.thumbnail_generation_finished")}</p>

            {corruptedWallpapers.length > 0 && (
              <Flex vertical gap="8px">
                <p style={{ color: "var(--color-red-900)", fontWeight: "bold" }}>
                  {t("wall.corrupted_wallpapers_message")}
                </p>
                <List
                  size="small"
                  bordered
                  dataSource={corruptedWallpapers}
                  renderItem={(wallpaper) => {
                    const displayName = typeof wallpaper.metadata.displayName === "string"
                      ? wallpaper.metadata.displayName
                      : wallpaper.metadata.displayName.en ||
                        Object.values(wallpaper.metadata.displayName)[0] ||
                        wallpaper.filename ||
                        "Unknown";
                    return <List.Item style={{ color: "var(--color-red-900)" }}>{displayName}</List.Item>;
                  }}
                />
              </Flex>
            )}

            <Flex justify="flex-end">
              <Button
                type="primary"
                onClick={() => {
                  open.value = false;
                }}
              >
                {t("close")}
              </Button>
            </Flex>
          </Flex>
        )}
    </Modal>
  );
}
