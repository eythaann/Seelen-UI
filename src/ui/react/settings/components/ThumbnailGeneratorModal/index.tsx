import { Flex, Modal, Progress } from "antd";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useSelector } from "react-redux";

import { newSelectors } from "../../modules/shared/store/app/reducer.ts";
import { generateThumbnails, getVideosWithoutThumbnail, type ThumbGenerationProgress } from "./thumbnailGenerator.ts";

export function ThumbnailGeneratorModal() {
  const { t } = useTranslation();
  const wallpapers = useSelector(newSelectors.wallpapers);

  const [thumbnailProgress, setThumbnailProgress] = useState<ThumbGenerationProgress | null>(null);

  useEffect(() => {
    // Do nothing if already processing
    if (thumbnailProgress !== null) {
      return;
    }

    const videosWithoutThumbnail = getVideosWithoutThumbnail(wallpapers);

    if (videosWithoutThumbnail.length > 0) {
      generateThumbnails(videosWithoutThumbnail, (progress) => {
        setThumbnailProgress(progress);
      }).finally(() => {
        setThumbnailProgress(null);
      });
    }
  }, [wallpapers, thumbnailProgress]);

  const percent = thumbnailProgress && thumbnailProgress.total > 0
    ? Math.round((thumbnailProgress.current / thumbnailProgress.total) * 100)
    : 0;

  return (
    <Modal
      open={thumbnailProgress !== null}
      title={t("wall.generating_thumbnails")}
      footer={null}
      closable={false}
      centered
    >
      <Flex vertical align="center" justify="center" gap="16px">
        <p>
          {t("wall.processing_video")} {`${(thumbnailProgress?.current || 0) + 1} / ${thumbnailProgress?.total || 0}`}
        </p>
        <p>{thumbnailProgress?.currentVideoName}</p>
        <Progress percent={percent} status="active" />
      </Flex>
    </Modal>
  );
}
