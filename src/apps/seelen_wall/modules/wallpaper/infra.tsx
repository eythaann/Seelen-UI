import { convertFileSrc } from '@tauri-apps/api/core';
import { ReactNode, RefObject, useEffect, useRef } from 'react';
import { useSelector } from 'react-redux';

import { Selectors } from '../shared/store/app';

export function ThemedWallpaper() {
  return <div className="wallpaper-empty" />;
}

export interface Props {
  path: string;
  containerRef: RefObject<HTMLDivElement>;
  onLoad: () => void;
  onError: () => void;
}

export function Wallpaper({ path, containerRef, onLoad, onError }: Props) {
  let stoped = useSelector(Selectors.stop);
  let wallpaper: ReactNode = null;

  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    if (videoRef.current) {
      if (stoped) {
        videoRef.current.pause();
      } else {
        videoRef.current.play();
      }
    }
  }, [stoped]);

  if (['.png', '.jpg', '.jpeg', '.webp', '.gif'].some((ext) => path.endsWith(ext))) {
    wallpaper = <img src={convertFileSrc(path)} onLoad={onLoad} onError={onError} />;
  }

  if (['.mp4', '.mkv', '.wav'].some((ext) => path.endsWith(ext))) {
    wallpaper = (
      <video
        ref={videoRef}
        src={convertFileSrc(path)}
        onLoadedData={onLoad}
        onError={onError}
        autoPlay={!stoped}
        loop
        muted
        playsInline
        disableRemotePlayback
      />
    );
  }

  return (
    <div ref={containerRef} className="wallpaper-container">
      {wallpaper}
    </div>
  );
}
