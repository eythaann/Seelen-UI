import { useSignalEffect } from '@preact/signals';
import { SeelenCommand } from '@seelen-ui/lib';
import { toPhysicalPixels } from '@shared';
import { cx } from '@shared/styles';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useCallback, useRef } from 'react';
import { useSelector } from 'react-redux';

import { Selectors } from '../../../shared/store/app';

import { $focused_app, $force_repositioning, $layout } from '../../../shared/state/mod';
import { ReservedContainer } from './reserved';

interface Props {
  hwnd: number;
  growFactor?: number;
}

export function Leaf({ hwnd, growFactor }: Props) {
  const reservation = useSelector(Selectors.reservation);
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

  useSignalEffect(() => {
    let _a = $layout.value;
    let _b = $force_repositioning.value;
    updateSize();
  });

  const isFocused = $focused_app.value.hwnd === hwnd;
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
