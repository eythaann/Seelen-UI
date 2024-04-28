import { toPhysicalPixels } from '../../../../../utils';
import { cx } from '../../../../../utils/styles';
import { ReservedContainer } from './reserved';
import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useRef } from 'react';
import { useSelector } from 'react-redux';

import { Selectors } from '../../../shared/store/app';

interface Props {
  hwnd: number;
  growFactor?: number;
}

export function LeafContainer({ hwnd, growFactor }: Props) {
  const ref = useRef<HTMLDivElement>(null);
  const reservation = useSelector(Selectors.reservation);
  const activeWindow = useSelector(Selectors.activeWindow);
  const borderSettings = useSelector(Selectors.settings.border);

  const updateSize = useCallback(() => {
    if (!ref.current) {
      return;
    }
    const domRect = ref.current.getBoundingClientRect();
    invoke('set_window_position', {
      hwnd: hwnd,
      rect: {
        top: toPhysicalPixels(domRect.top) + toPhysicalPixels(window.screenY),
        left: toPhysicalPixels(domRect.left) + toPhysicalPixels(window.screenX),
        right: toPhysicalPixels(domRect.width),
        bottom: toPhysicalPixels(domRect.height),
      },
    });
  }, [hwnd]);

  useEffect(updateSize);

  const isFocused = activeWindow === hwnd;
  return (
    <div
      ref={ref}
      style={{
        flexGrow: growFactor,
      }}
      className={cx('wm-container', 'wm-leaf', {
        'wm-leaf-focused': isFocused,
        'wm-leaf-with-borders': borderSettings.enabled,
      })}
    >
      {!!reservation && isFocused && <ReservedContainer reservation={reservation} />}
    </div>
  );
}
