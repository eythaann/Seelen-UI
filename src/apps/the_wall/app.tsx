import { ErrorBoundary } from '../seelenweg/components/Error';
import { convertFileSrc } from '@tauri-apps/api/core';
import { getCurrent } from '@tauri-apps/api/webviewWindow';
import { useEffect } from 'react';

export function App() {
  useEffect(() => {
    getCurrent().show();
  }, []);

  let url = convertFileSrc(
    'C:\\Users\\dlmqc\\source\\repos\\seelen-ui\\target\\debug\\static\\wallpaper.mp4',
  );

  return (
    <ErrorBoundary fallback={<div>Error</div>}>
      <video
        width="100%"
        height="100%"
        autoPlay
        muted
        loop
        style={{
          objectFit: 'fill',
        }}
      >
        <source src={url} type="video/mp4" />
      </video>
    </ErrorBoundary>
  );
}
