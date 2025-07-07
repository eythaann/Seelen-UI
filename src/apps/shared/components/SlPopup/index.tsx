import { useSignal, useSignalEffect } from '@preact/signals';
import { useSignalRef } from '@preact/signals/utils';
import { useDebounce } from '@shared/hooks';
import { $is_this_webview_focused } from '@shared/signals';
import { cx } from '@shared/styles';
import { cloneElement, ComponentChild, VNode } from 'preact';
import { createPortal, CSSProperties, useCallback, useEffect, useRef } from 'preact/compat';

import { LegacyCustomAnimationProps } from '../AnimatedWrappers/domain';

import { calculateElementPosition } from './positioning';

export interface SlPopupProps {
  animationDescription?: LegacyCustomAnimationProps;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  content: ComponentChild;
  children: VNode<any>;
  placement?: 'bottom' | 'top' | 'left' | 'right';
  trigger?: 'click' | 'hover' | 'manual';
  mouseEnterDelay?: number;
}

export function SlPopup(props: SlPopupProps) {
  const {
    open: openProp,
    onOpenChange: onOpenChangeProp,
    content,
    children: trigger,
    trigger: triggerType = 'click',
    mouseEnterDelay = 0.4,
    animationDescription = {},
    placement: preferredPosition = 'bottom',
  } = props;
  const { openAnimationName, closeAnimationName } = animationDescription;

  const unique_id = useRef(crypto.randomUUID());

  const $was_open = useSignal(false);
  const $is_open = useSignal(openProp);
  const $popup_position_styles = useSignal<CSSProperties>({});

  const $triggerRef = useSignalRef<HTMLElement | null>(null);
  const popupRef = useRef<HTMLDivElement>(null);

  const mouseDelayedAction = useDebounce((cb: () => void) => cb(), mouseEnterDelay * 1000);
  const onOpenChange = useCallback(
    (open: boolean) => {
      if (openProp !== undefined) {
        $is_open.value = open;
      }
      onOpenChangeProp?.(open);
    },
    [onOpenChangeProp, openProp !== undefined],
  );

  useEffect(() => {
    return () => {
      mouseDelayedAction.cancel();
    };
  }, []);

  useEffect(() => {
    const cb = (e: MouseEvent) => {
      const clickedElement = e.target as HTMLElement;
      const isTrigger = clickedElement.closest(`[data-popup-id="${unique_id.current}"]`);
      const isPopup = clickedElement.closest('.sl-popup');

      if (!isTrigger && !isPopup && $is_open.value) {
        onOpenChange(false);
      }
    };
    window.addEventListener('click', cb);

    return () => {
      window.removeEventListener('click', cb);
    };
  }, [onOpenChange]);

  useSignalEffect(() => {
    if (!$is_this_webview_focused.value) {
      onOpenChange(false);
    }
  });

  useEffect(() => {
    const newValue = openProp ?? $is_open.value;
    if (newValue !== $is_open.value) {
      $is_open.value = newValue;
    }
  }, [openProp]);

  useSignalEffect(() => {
    if ($is_open.value && !$was_open.peek()) {
      $was_open.value = true;
    }
  });

  const updatePopupPosition = () => {
    if (!$was_open.value || !$triggerRef.current || !popupRef.current) return;

    const position = calculateElementPosition(
      $triggerRef.current,
      popupRef.current,
      preferredPosition,
    );

    const newStyles = {
      position: 'fixed',
      top: `${position.top}px`,
      left: `${position.left}px`,
      zIndex: 1000,
    };

    // Only update if styles actually changed
    if (JSON.stringify(newStyles) !== JSON.stringify($popup_position_styles.peek())) {
      $popup_position_styles.value = newStyles;
    }
  };

  useSignalEffect(updatePopupPosition);

  function onMouseEnter() {
    if (triggerType === 'hover') {
      if ($is_open.value) {
        mouseDelayedAction.cancel();
        return;
      }
      mouseDelayedAction(() => onOpenChange(true));
    }
  }

  function onMouseLeave() {
    if (triggerType === 'hover') {
      if (!$is_open.value) {
        mouseDelayedAction.cancel();
        return;
      }
      mouseDelayedAction(() => onOpenChange(false));
    }
  }

  return (
    <>
      {cloneElement(trigger, {
        'data-popup-id': unique_id.current,
        onClick() {
          if (triggerType === 'click') {
            onOpenChange(!$is_open.value);
          }
          trigger.props.onClick?.();
        },
        onMouseEnter,
        onMouseLeave,
        ref(_element: HTMLElement) {
          const element = document.querySelector(`[data-popup-id="${unique_id.current}"]`);
          if (element && $triggerRef.current !== element) {
            $triggerRef.current = element as HTMLElement;
          }
          trigger.props.ref?.(_element);
        },
      })}
      {$was_open.value &&
        createPortal(
          <div>
            <div
              ref={popupRef}
              onMouseEnter={onMouseEnter}
              onMouseLeave={onMouseLeave}
              style={{
                display: $is_open.value ? 'block' : 'none',
                height: 'min-content',
                width: 'min-content',
                ...$popup_position_styles.value,
              }}
              className={cx('sl-popup', {
                'sl-popup-open': $is_open.value,
                'sl-popup-closed': !$is_open.value,
                [openAnimationName ?? '!?']: openAnimationName && $is_open.value,
                [closeAnimationName ?? '!?']: closeAnimationName && !$is_open.value,
              })}
            >
              {content}
            </div>
          </div>,
          document.body,
        )}
    </>
  );
}
