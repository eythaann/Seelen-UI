import { getCurrent } from '@tauri-apps/api/webviewWindow';
import { Update } from '@tauri-apps/plugin-updater';
import { Button } from 'antd';

interface Props {
  update: Update;
}

export function UpdateModal({ update }: Props) {
  const onDownload = async () => {
    await update.downloadAndInstall();
  };

  return (
    <>
      <div className="title">
        Update for <span className="package">Komorebi UI</span> available!
      </div>
      <div className="description">
        <div>
          <b>Date:</b> {update.date ? update.date.replace(/\s.*/, '') : '-'}
        </div>
        <div>
          <b>Version:</b> {update.version}
        </div>
        <br/>
        <p>
          <b>
            To read the changelog visit the {' '}
            <a
              href={`https://github.com/eythaann/komorebi-ui/releases/tag/v${update.version}`}
              target="_blank"
            >
              Github Release Page
            </a>
          </b>
        </p>
      </div>
      <div className="footer">
        <Button onClick={() => getCurrent().close()}>Later</Button>
        <Button onClick={onDownload} type="primary">Download & Install</Button>
      </div>
    </>
  );
}
