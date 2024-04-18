import { toPhysicalPixels } from '../../../../../utils';
import { cx } from '../../../../../utils/styles';
import { ReservedContainer } from './reserved';
import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useRef } from 'react';
import { useSelector } from 'react-redux';

import { Selectors } from '../../../shared/store/app';

import cs from '../index.module.css';

interface Props {
  hwnd: number;
  growFactor?: number;
}

export function LeafContainer({ hwnd, growFactor }: Props) {
  const ref = useRef<HTMLDivElement>(null);
  const reservation = useSelector(Selectors.reservation);
  const activeWindow = useSelector(Selectors.activeWindow);

  const updateSize = useCallback(() => {
    if (!ref.current) {
      return;
    }
    const domRect = ref.current.getBoundingClientRect();
    invoke('set_window_position', {
      hwnd: hwnd,
      rect: {
        top: toPhysicalPixels(domRect.top),
        left: toPhysicalPixels(domRect.left),
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
      className={cx(cs.container, cs.leaf, {
        [cs.focused!]: isFocused,
      })}
    >
      {!!reservation && isFocused && <ReservedContainer reservation={reservation} />}
    </div>
  );
}
