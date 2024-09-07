import { convertFileSrc } from '@tauri-apps/api/core';
import { ReactNode } from 'react';

const path = 'C:\\Users\\dlmqc\\Downloads\\white-moonflower-field-katana-moewalls-com.mp4';

export function App() {
  let wallpaper: ReactNode = null;

  if (path.endsWith('.mp4')) {
    wallpaper = (
      <video
        style={{ width: '100%', height: '100%', objectFit: 'cover' }}
        src={convertFileSrc(path)}
        autoPlay
        loop
        muted
        playsInline
        disableRemotePlayback
      />
    );
  }

  return (
    <>
      {!wallpaper && <div className="wallpaper-empty">Seelen UI - No wallpaper</div>}
      {wallpaper}
    </>
  );
}
