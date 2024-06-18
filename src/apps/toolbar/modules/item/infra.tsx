import {
  exposedIcons,
  exposedIconsRegex,
  Icon,
  isValidIconName,
} from '../../../shared/components/Icon';
import { ToolbarModule } from '../../../shared/schemas/Placeholders';
import { cx } from '../../../shared/styles';
import { Tooltip } from 'antd';
import { cloneDeep } from 'lodash';
import { evaluate } from 'mathjs';
import React, { useEffect, useRef } from 'react';
import { useSelector } from 'react-redux';

import { Selectors } from '../shared/store/app';
import { performClick } from './app';

interface Props {
  module: ToolbarModule;
  extraVars?: Record<string, any>;
  active?: boolean;
}

class Scope {
  scope: Map<string, any>;

  constructor() {
    this.scope = new Map();
  }

  get(key: string) {
    return this.scope.get(key);
  }

  set(key: string, value: any) {
    return this.scope.set(key, value);
  }

  has(key: string) {
    return this.scope.has(key);
  }

  keys(): string[] | IterableIterator<string> {
    return this.scope.keys();
  }
}

export function ElementsFromEvaluated(content: any) {
  if (typeof content !== 'string') {
    content = JSON.stringify(content);
  }

  const parts: string[] = content.split(exposedIconsRegex);
  const result: React.ReactNode[] = [];

  parts.forEach((part: string, index: number) => {
    if (isValidIconName(part)) {
      const [iconName, size] = part.split(':') as [Icon, string?];
      result.push(
        <Icon
          key={index}
          iconName={iconName}
          propsIcon={{ size: size ? size + 'px' : undefined }}
        />,
      );
    } else if (part) {
      result.push(<span key={index}>{part}</span>);
    }
  });

  return result;
}

export function Item({ extraVars, module, active }: Props) {
  const { template, tooltip, onClick } = module;

  const [mounted, setMounted] = React.useState(false);
  const env = useSelector(Selectors.env);
  const window = useSelector(Selectors.focused) || {
    name: 'None',
    title: 'No Window Focused',
  };

  const scope = useRef(new Scope());

  useEffect(() => {
    scope.current.set('icon', cloneDeep(exposedIcons));
    scope.current.set('env', cloneDeep(env));
    setMounted(true);
  }, []);

  scope.current.set('window', { ...window });
  if (extraVars) {
    Object.keys(extraVars).forEach((key) => {
      scope.current.set(key, extraVars[key]);
    });
  }

  if (!mounted) {
    return null;
  }

  const elements = ElementsFromEvaluated(evaluate(template, scope.current));
  if (!elements.length) {
    return null;
  }

  return (
    <Tooltip
      arrow={false}
      mouseLeaveDelay={0}
      overlayClassName="ft-bar-item-tooltip"
      title={tooltip ? ElementsFromEvaluated(evaluate(tooltip, scope.current)) : undefined}
    >
      <div
        onClick={() => performClick(onClick, scope.current)}
        className={cx('ft-bar-item', {
          'ft-bar-item-clickable': !!onClick,
          'ft-bar-item-active': active,
        })}
      >
        <div className="ft-bar-item-content">{elements}</div>
      </div>
    </Tooltip>
  );
}
