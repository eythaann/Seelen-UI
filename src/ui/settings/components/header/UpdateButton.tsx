import { invoke, SeelenCommand } from '@seelen-ui/lib';
import { Icon } from '@shared/components/Icon';
import { Badge, Button, Tooltip } from 'antd';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

export function UpdateButton() {
  const [downloading, setDownloading] = useState<boolean>(false);
  const [update, setUpdate] = useState<boolean>(false);

  const { t } = useTranslation();

  useEffect(() => {
    invoke(SeelenCommand.CheckForUpdates)
      .then(setUpdate)
      .catch(() => setUpdate(false));
  }, []);

  if (!update) {
    return null;
  }

  return (
    <Tooltip title={downloading ? t('update.downloading') : t('update.available')}>
      <Button
        type="text"
        loading={downloading}
        onClick={() => {
          if (!downloading) {
            setDownloading(true);
            invoke(SeelenCommand.InstallLastAvailableUpdate).finally(() => setDownloading(false));
          }
        }}
      >
        <Badge dot>
          <Icon iconName="TbDownload" />
        </Badge>
      </Button>
    </Tooltip>
  );
}
