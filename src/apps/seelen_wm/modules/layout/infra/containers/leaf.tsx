import { toPhysicalPixels } from '../../../../../shared';
import { cx } from '../../../../../shared/styles';
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

    const border = borderSettings.enabled ? borderSettings.width + borderSettings.offset : 0;
    const domRect = ref.current.getBoundingClientRect();
    const top = domRect.top + window.screenY + border;
    const left = domRect.left + window.screenX + border;
    invoke('set_window_position', {
      hwnd: hwnd,
      rect: {
        top: toPhysicalPixels(top),
        left: toPhysicalPixels(left),
        right: toPhysicalPixels(left + domRect.width - border * 2),
        bottom: toPhysicalPixels(top + domRect.height - border * 2),
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
