import { useSignal, useSignalEffect } from '@preact/signals';
import { useSignalRef } from '@preact/signals/utils';
import { useDebounce } from '@shared/hooks';
import { $is_this_webview_focused } from '@shared/signals';
import { cx } from '@shared/styles';
import { cloneElement, ComponentChild, VNode } from 'preact';
import { JSX } from 'preact/compat';
import {
  createPortal,
  CSSProperties,
  HTMLAttributes,
  useCallback,
  useEffect,
  useRef,
} from 'preact/compat';

import { LegacyCustomAnimationProps } from '../AnimatedWrappers/domain';

import { calculateElementPosition } from './positioning';

import './base.css';

type BasicElementProps = HTMLAttributes<HTMLElement> & { [x in `data-${string}`]: string };

export interface SlPopupProps<TriggerProps extends BasicElementProps> extends BasicElementProps {
  debug?: boolean;
  animationDescription?: LegacyCustomAnimationProps;
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  content: ComponentChild;
  children: VNode<TriggerProps>;
  placement?: 'bottom' | 'top' | 'left' | 'right';
  trigger?: 'click' | 'hover' | 'manual';
  mouseEnterDelay?: number;
}

export function SlPopup<TProps extends BasicElementProps>(props: SlPopupProps<TProps>) {
  const {
    open: openProp,
    debug: _debug,
    onOpenChange: onOpenChangeProp,
    content,
    children: trigger,
    trigger: triggerType = 'click',
    mouseEnterDelay = 0.4,
    animationDescription = {},
    placement: preferredPosition = 'bottom',
    ...rest
  } = props;
  const { openAnimationName, closeAnimationName } = animationDescription;
  const isExternallyHandled = openProp !== undefined;

  const unique_id = useRef(crypto.randomUUID());

  const $was_open = useSignal(false);
  const $is_open = useSignal(openProp);
  const $popup_position_styles = useSignal<CSSProperties>({});

  const $triggerRef = useSignalRef<HTMLElement | null>(null);
  const popupRef = useRef<HTMLDivElement>(null);

  const mouseDelayedAction = useDebounce((cb: () => void) => cb(), mouseEnterDelay * 1000);
  const onOpenChange = useCallback(
    (open: boolean) => {
      if (!isExternallyHandled) {
        $is_open.value = open;
      }
      onOpenChangeProp?.(open);
    },
    [onOpenChangeProp, isExternallyHandled],
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
    if (!$was_open.value || !$is_open.value || !$triggerRef.current || !popupRef.current) return;

    const position = calculateElementPosition(
      $triggerRef.current,
      popupRef.current,
      preferredPosition,
    );

    const newStyles = {
      top: `${position.top}px`,
      left: `${position.left}px`,
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

  const { className: _className, ...toForwardDown } = rest;
  const triggerProps = {
    ...toForwardDown,
    ...trigger.props,
    'data-popup-id': unique_id.current as string,
    onClick(e: JSX.TargetedMouseEvent<HTMLElement>) {
      console.log('clicked');
      trigger.props.onClick?.(e);
      console.log('clicked2');
      if (triggerType === 'click') {
        onOpenChange(!$is_open.value);
      }
      toForwardDown.onClick?.(e);
    },
    onMouseEnter(e: JSX.TargetedMouseEvent<HTMLElement>) {
      trigger.props.onMouseEnter?.(e);
      if (triggerType === 'hover') {
        onMouseEnter();
      }
      toForwardDown.onMouseEnter?.(e);
    },
    onMouseLeave(e: JSX.TargetedMouseEvent<HTMLElement>) {
      trigger.props.onMouseLeave?.(e);
      if (triggerType === 'hover') {
        onMouseLeave();
      }
      toForwardDown.onMouseLeave?.(e);
    },
    ref(_element: HTMLElement) {
      const element = document.querySelector(`[data-popup-id="${unique_id.current}"]`);
      if (element && $triggerRef.current !== element) {
        $triggerRef.current = element as HTMLElement;
      }
    },
  };

  return (
    <>
      {cloneElement(trigger, triggerProps)}
      {$was_open.value &&
        createPortal(
          <div>
            <div
              ref={popupRef}
              onMouseEnter={onMouseEnter}
              onMouseLeave={onMouseLeave}
              style={{
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
