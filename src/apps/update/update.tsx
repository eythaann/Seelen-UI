import { getCurrent } from '@tauri-apps/api/webviewWindow';
import { Update } from '@tauri-apps/plugin-updater';
import { Button, Progress } from 'antd';
import { useState } from 'react';

interface Props {
  update: Update;
}

export function UpdateModal({ update }: Props) {
  const [total, setTotal] = useState<number | null>(null);
  const [current, setCurrent] = useState<number>(0);

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
    return (
      <>
        <div className="title">
          Downloading <span className="package">Seelen UI</span> available!
        </div>
        <div className="description">
          <Progress
            className="progress"
            type="dashboard"
            percent={Math.floor((current / total) * 100)}
            strokeWidth={12}
            strokeColor={[
              '#0054b6',
              '00670f',
            ]}
          />
        </div>
      </>
    );
  }

  return (
    <>
      <div className="title">
        Update for <span className="package">Seelen UI</span> available!
      </div>
      <div className="description">
        <div>
          <b>Date:</b> {update.date ? update.date.replace(/\s.*/, '') : '-'}
        </div>
        <div>
          <b>Version:</b> {update.version}
        </div>
        <br />
        <p>
          <b>
            To read the changelog visit the{' '}
            <a
              href={`https://github.com/eythaann/seelen-ui/releases/tag/v${update.version}`}
              target="_blank"
            >
              Github Release Page
            </a>
          </b>
        </p>
      </div>
      <div className="footer">
        <Button onClick={() => getCurrent().close()}>Later</Button>
        <Button onClick={onDownload} type="primary">
          Download & Install
        </Button>
      </div>
    </>
  );
}
