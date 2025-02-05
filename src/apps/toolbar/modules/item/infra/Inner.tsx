import { ToolbarItem } from '@seelen-ui/lib/types';
import { Tooltip } from 'antd';
import { Reorder } from 'framer-motion';
import { cloneDeep } from 'lodash';
import { evaluate, isResultSet } from 'mathjs';
import React, { PropsWithChildren, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { Selectors } from '../../shared/store/app';
import { safeEval, Scope } from '../app';

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

export function ElementsFromEvaluated(content: any) {
  let text: string = '';

  if (typeof content === 'string') {
    text = content;
  } else if (isResultSet(content)) {
    text = content.entries.reduce((acc: string, current: any) => {
      return `${acc}${typeof current === 'string' ? current : JSON.stringify(current)}`;
    }, '');
  } else {
    text = JSON.stringify(content);
  }

  const parts: string[] = text.split(/\[|\]/g).filter((part: string) => part);
  const result: React.ReactNode[] = parts.map((part: string, index: number) => {
    return <StringToElement key={index} text={part} />;
  });

  return result;
}

export function InnerItem(props: InnerItemProps) {
  const {
    extraVars,
    module,
    active,
    onClick: onClickProp,
    onKeydown: onKeydownProp,
    onWheel: onWheelProp,
    children,
    clickable = true,
    ...rest
  } = props;
  const { template, tooltip, onClickV2, style, id, badge } = module;

  const [mounted, setMounted] = React.useState(false);
  const env = useSelector(Selectors.env);

  const { t } = useTranslation();
  const scope = useRef(new Scope());

  useEffect(() => {
    scope.current.loadInvokeActions();

    scope.current.set('env', cloneDeep(env));

    scope.current.set('getIcon', StringToElement.getIcon);
    scope.current.set('imgFromUrl', StringToElement.imgFromUrl);
    scope.current.set('imgFromPath', StringToElement.imgFromPath);
    scope.current.set('imgFromExe', StringToElement.imgFromExe);

    setMounted(true);
  }, []);

  if (!mounted) {
    return null;
  }

  scope.current.set('t', t);
  if (extraVars) {
    Object.keys(extraVars).forEach((key) => {
      scope.current.set(key, extraVars[key]);
    });
  }

  function parseStringToElements(text: string) {
    /// backward compatibility with v1 icon object
    let expr = text.replaceAll(/icon\.(\w+)/g, 'getIcon("$1")');
    return ElementsFromEvaluated(evaluate(expr, scope.current));
  }

  const elements = template ? parseStringToElements(template) : [];
  if (!elements.length && !children) {
    return null;
  }

  const badgeContent = badge ? parseStringToElements(badge) : null;

  return (
    <Tooltip
      arrow={false}
      mouseLeaveDelay={0}
      classNames={{ root: 'ft-bar-item-tooltip' }}
      title={tooltip ? parseStringToElements(tooltip) : undefined}
    >
      <Reorder.Item
        {...rest}
        id={id}
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
            safeEval(onClickV2, scope.current);
          }
        }}
        as="div"
        transition={{ duration: 0.15 }}
        onContextMenu={(e) => {
          e.stopPropagation();
          (rest as any).onContextMenu?.(e);
        }}
      >
        <div className="ft-bar-item-content">
          {children || elements}
          {!!badgeContent?.length && <div className="ft-bar-item-badge">{badgeContent}</div>}
        </div>
      </Reorder.Item>
    </Tooltip>
  );
}
