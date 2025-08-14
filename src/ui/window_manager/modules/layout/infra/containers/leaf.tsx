import { SeelenCommand } from '@seelen-ui/lib';
import { toPhysicalPixels } from '@shared';
import { cx } from '@shared/styles';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useCallback, useEffect, useRef } from 'react';
import { useSelector } from 'react-redux';

import { Selectors } from '../../../shared/store/app';

import { ReservedContainer } from './reserved';

interface Props {
  hwnd: number;
  growFactor?: number;
}

export function Leaf({ hwnd, growFactor }: Props) {
  const _version = useSelector(Selectors._version);

  const reservation = useSelector(Selectors.reservation);
  const activeWindow = useSelector(Selectors.activeWindow);
  const borderSettings = useSelector(Selectors.settings.border);

  const ref = useRef<HTMLDivElement>(null);

  const updateSize = useCallback(async () => {
    if (!ref.current) {
      return;
    }

    const border = borderSettings.enabled ? borderSettings.width + borderSettings.offset : 0;
    const domRect = ref.current.getBoundingClientRect();
    const { x: windowX, y: windowY } = await getCurrentWindow().outerPosition();
    const top = windowY + toPhysicalPixels(domRect.top + border);
    const left = windowX + toPhysicalPixels(domRect.left + border);
    invoke(SeelenCommand.SetWindowPosition, {
      hwnd: hwnd,
      rect: {
        top,
        left,
        right: left + toPhysicalPixels(domRect.width - border * 2),
        bottom: top + toPhysicalPixels(domRect.height - border * 2),
      },
    });
  }, [hwnd]);

  useEffect(() => {
    updateSize();
  });

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
