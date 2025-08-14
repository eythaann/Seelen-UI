import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { useComputed } from '@preact/signals';
import { RemoteDataDeclaration, ToolbarItem } from '@seelen-ui/lib/types';
import { useDeepCompareEffect } from '@shared/hooks';
import { Tooltip } from 'antd';
import { HTMLAttributes, PropsWithChildren, useEffect, useRef, useState } from 'preact/compat';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { Selectors } from '../../shared/store/app';
import { EvaluateAction } from '../app';

import { cx } from '../../../../shared/styles';
import { $toolbar_state } from '../../shared/state/items';
import { SanboxedComponent } from './EvaluatedComponents';

export interface InnerItemProps extends PropsWithChildren {
  module: Omit<ToolbarItem, 'type'>;
  extraVars?: Record<string, any>;
  active?: boolean;
  clickable?: boolean;
  onWheel?: (e: WheelEvent) => void;
  // needed for dropdown/popup wrappers
  onClick?: (e: MouseEvent) => void;
  onKeydown?: (e: KeyboardEvent) => void;
}

export function InnerItem(props: InnerItemProps) {
  const {
    extraVars = {},
    module,
    active,
    onClick: onClickProp,
    onKeydown: onKeydownProp,
    onWheel: onWheelProp,
    children,
    clickable = true,
    ...rest
  } = props;

  const { template, tooltip, onClickV2, style, id, badge, remoteData = {} } = module;

  const fetchedData = useRemoteData(remoteData);
  const isReorderDisabled = useComputed(() => $toolbar_state.value.isReorderDisabled);
  const env = useSelector(Selectors.env);

  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id,
    disabled: isReorderDisabled.value,
    animateLayoutChanges: () => false,
  });

  const { t } = useTranslation();

  const [scope, setScope] = useState<Record<string, any>>({
    env,
    t,
    ...extraVars,
    ...fetchedData,
  });

  useEffect(() => {
    setScope((s) => ({ ...s, env, t }));
  }, [env, t]);

  useDeepCompareEffect(() => {
    setScope((s) => ({ ...s, ...extraVars, ...fetchedData }));
  }, [extraVars, fetchedData]);

  return (
    <Tooltip
      arrow={false}
      mouseLeaveDelay={0}
      classNames={{ root: 'ft-bar-item-tooltip' }}
      title={tooltip ? <SanboxedComponent code={tooltip} scope={scope} /> : undefined}
    >
      <div
        ref={setNodeRef}
        {...listeners}
        {...(attributes as HTMLAttributes<HTMLDivElement>)}
        {...rest}
        id={id}
        style={{ ...style, transform: CSS.Translate.toString(transform), transition, opacity: isDragging ? 0.3 : 1 }}
        className={cx('ft-bar-item', {
          // onClickProp is omitted cuz it always comes via context menu dropdown wrapper
          'ft-bar-item-clickable': clickable || onClickV2,
          'ft-bar-item-active': active,
        })}
        onWheel={onWheelProp}
        onKeyDown={onKeydownProp}
        onClick={(e: MouseEvent) => {
          onClickProp?.(e);
          if (onClickV2) {
            EvaluateAction(onClickV2, scope);
          }
        }}
        onContextMenu={(e: MouseEvent) => {
          e.stopPropagation();
          (rest as any).onContextMenu?.(e);
        }}
      >
        <div className="ft-bar-item-content">
          {children || <SanboxedComponent code={template} scope={scope} />}
          {!!badge && (
            <div className="ft-bar-item-badge">
              <SanboxedComponent code={badge} scope={scope} />
            </div>
          )}
        </div>
      </div>
    </Tooltip>
  );
}

function useRemoteData(remoteData: Record<string, RemoteDataDeclaration | undefined>) {
  const [state, setState] = useState<Record<string, any>>(() => {
    return Object.keys(remoteData).reduce((acc, key) => ({ ...acc, [key]: undefined }), {});
  });

  const intervalsRef = useRef<Record<string, number>>({});
  const mountedRef = useRef(true);

  const fetchData = async (key: string, rd: RemoteDataDeclaration): Promise<void> => {
    if (!mountedRef.current) return;

    try {
      const response = await fetch(rd.url, rd.requestInit as RequestInit);
      const data = response.headers.get('Content-Type')?.includes('application/json')
        ? await response.json()
        : await response.text();

      if (mountedRef.current) {
        setState((prev) => ({
          ...prev,
          [key]: data,
        }));
      }
    } catch (error) {
      console.error(`Error fetching ${key}:`, error);
    }
  };

  useDeepCompareEffect(() => {
    mountedRef.current = true;
    Object.values(intervalsRef.current).forEach(clearInterval);
    intervalsRef.current = {};

    const initialState = Object.keys(remoteData).reduce(
      (acc, key) => ({ ...acc, [key]: undefined }),
      {},
    );

    setState((prev) => ({ ...initialState, ...prev }));

    Object.entries(remoteData).forEach(([key, rd]) => {
      if (!rd) return;
      fetchData(key, rd);
      if (rd.updateIntervalSeconds) {
        intervalsRef.current[key] = window.setInterval(
          () => fetchData(key, rd),
          rd.updateIntervalSeconds * 1000,
        );
      }
    });

    return () => {
      mountedRef.current = false;
      Object.values(intervalsRef.current).forEach(clearInterval);
    };
  }, [remoteData]);

  return state;
}
