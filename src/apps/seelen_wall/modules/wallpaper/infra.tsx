import { convertFileSrc } from '@tauri-apps/api/core';
import { ReactNode } from 'react';

export function Wallpaper({ path }: { path: string }) {
  let wallpaper: ReactNode = null;
  if (['.png', '.jpg', '.jpeg', '.gif'].some((ext) => path.endsWith(ext))) {
    wallpaper = (
      <img
        style={{ width: '100%', height: '100%', objectFit: 'cover' }}
        src={convertFileSrc(path)}
      />
    );
  }

  if (['.mp4', '.mkv', '.wab'].some((ext) => path.endsWith(ext))) {
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
      {!wallpaper && <div className="wallpaper-empty" />}
      {wallpaper}
    </>
  );
}
