import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { Update } from '@tauri-apps/plugin-updater';
import { Button, Progress } from 'antd';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

interface Props {
  update: Update;
}

export function UpdateModal({ update }: Props) {
  const [total, setTotal] = useState<number | null>(null);
  const [current, setCurrent] = useState<number>(0);

  const { t } = useTranslation();

  const onDownload = async () => {
    update.downloadAndInstall((progress) => {
      if (progress.event === 'Started' && progress.data.contentLength) {
        setTotal(progress.data.contentLength);
      }

      if (progress.event === 'Progress') {
        setCurrent((v) => v + progress.data.chunkLength * 1000);
      }

      if (progress.event === 'Finished') {
        setCurrent(total!);
      }
    });
  };

  if (total != null) {
    const percent = Math.floor((current / total) * 100);
    return (
      <>
        <div className="title">
          {percent === 100 ? t('update.installing') : t('update.downloading')}
        </div>
        <div className="description">
          <Progress
            className="progress"
            type="dashboard"
            percent={percent}
            strokeWidth={12}
            strokeColor={['#0054b6', '00670f']}
          />
        </div>
      </>
    );
  }

  return (
    <>
      <div className="title">{t('update.title')}</div>
      <div className="description">
        <div>
          <b>{t('update.date')}:</b> {update.date ? update.date.replace(/\s.*/, '') : '-'}
        </div>
        <div>
          <b>{t('update.version')}:</b> {update.version}
        </div>
        <br />
        <p>
          <b>
            {t('update.extra_info')}:{' '}
            <a
              href={`https://github.com/eythaann/seelen-ui/releases/tag/v${update.version}`}
              target="_blank"
            >
              {t('update.page')}
            </a>
          </b>
        </p>
      </div>
      <div className="footer">
        <Button onClick={() => getCurrentWebviewWindow().close()}>{t('update.cancel')}</Button>
        <Button onClick={onDownload} type="primary">
          {t('update.download')}
        </Button>
      </div>
    </>
  );
}
