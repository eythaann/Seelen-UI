import {
  exposedIcons,
  exposedIconsRegex,
  Icon,
  isValidIconName,
} from '../../../utils/components/Icon';
import { ToolbarModule } from '../../../utils/schemas/Placeholders';
import { cx } from '../../../utils/styles';
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

  const parts = content.split(exposedIconsRegex);
  return parts.map((part: string, index: number) => {
    if (isValidIconName(part)) {
      const [iconName, size] = part.split(':') as [Icon, string?];
      return (
        <Icon
          key={index}
          iconName={iconName}
          propsIcon={{ size: size ? size + 'px' : undefined }}
        />
      );
    } else {
      return <React.Fragment key={index}>{part}</React.Fragment>;
    }
  });
}

export function Item({ extraVars, module }: Props) {
  const { template, tooltip, onClick } = module;
  const env = useSelector(Selectors.env);
  const window = useSelector(Selectors.focused);
  const scope = useRef(new Scope());

  useEffect(() => {
    scope.current.set('icon', cloneDeep(exposedIcons));
    scope.current.set('env', cloneDeep(env));
  }, []);

  if (!window) {
    return null;
  }

  scope.current.set('window', { ...window });
  if (extraVars) {
    Object.keys(extraVars).forEach((key) => {
      scope.current.set(key, extraVars[key]);
    });
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
        })}
      >
        {ElementsFromEvaluated(evaluate(template, scope.current))}
      </div>
    </Tooltip>
  );
}
