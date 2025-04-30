import { RemoteDataDeclaration, ToolbarItem } from '@seelen-ui/lib/types';
import useDeepCompareEffect from '@shared/hooks';
import { Tooltip } from 'antd';
import { Reorder } from 'framer-motion';
import React, { PropsWithChildren, useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { Selectors } from '../../shared/store/app';
import { EvaluateAction, SanboxedComponent } from '../app';

import { cx } from '../../../../shared/styles';
import { StringToElement } from './StringElement';

export interface InnerItemProps extends PropsWithChildren {
  module: Omit<ToolbarItem, 'type'>;
  extraVars?: Record<string, any>;
  active?: boolean;
  clickable?: boolean;
  onWheel?: (e: React.WheelEvent) => void;
  // needed for dropdown/popup wrappers
  onClick?: (e: React.MouseEvent) => void;
  onKeydown?: (e: React.KeyboardEvent) => void;
}

const commonScope = {
  button: StringToElement.getButton,
  icon: StringToElement.getIcon,
  getIcon: StringToElement.getIcon,
  imgFromUrl: StringToElement.imgFromUrl,
  imgFromPath: StringToElement.imgFromPath,
  imgFromExe: StringToElement.imgFromApp, // backward compatibility
  imgFromApp: (opt: { path?: string | null; umid?: string | null; size?: number }) => {
    return StringToElement.imgFromApp(opt.path, opt.umid, opt.size);
  },
};

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
  const isReorderDisabled = useSelector(Selectors.items.isReorderDisabled);
  const env = useSelector(Selectors.env);

  const { t } = useTranslation();

  const [scope, setScope] = useState<Record<string, any>>({
    ...commonScope,
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
    <Reorder.Item
      {...rest}
      id={id}
      drag={!isReorderDisabled}
      value={(module as any).__value__ || module}
      style={style}
      className={cx('ft-bar-item', {
        // onClickProp is omitted cuz it always comes via context menu dropdown wrapper
        'ft-bar-item-clickable': clickable || onClickV2,
        'ft-bar-item-active': active,
      })}
      onWheel={onWheelProp}
      onKeyDown={onKeydownProp}
      onClick={(e) => {
        onClickProp?.(e);
        if (onClickV2) {
          EvaluateAction(onClickV2, scope);
        }
      }}
      as="div"
      transition={{ duration: 0.15 }}
      onContextMenu={(e) => {
        e.stopPropagation();
        (rest as any).onContextMenu?.(e);
      }}
    >
      <Tooltip
        arrow={false}
        mouseLeaveDelay={0}
        classNames={{ root: 'ft-bar-item-tooltip' }}
        title={tooltip ? <SanboxedComponent code={tooltip} scope={scope} /> : undefined}
      >
        <div className="ft-bar-item-content">
          {children || <SanboxedComponent code={template} scope={scope} />}
          {!!badge && (
            <div className="ft-bar-item-badge">
              <SanboxedComponent code={badge} scope={scope} />
            </div>
          )}
        </div>
      </Tooltip>
    </Reorder.Item>
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
