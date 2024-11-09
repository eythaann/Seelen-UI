import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { Tooltip } from 'antd';
import { Reorder } from 'framer-motion';
import { cloneDeep } from 'lodash';
import { evaluate, isResultSet } from 'mathjs';
import React, { PropsWithChildren, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';
import { ToolbarModule } from 'seelen-core';

import { LAZY_CONSTANTS } from '../../shared/utils/infra';

import { Selectors } from '../../shared/store/app';
import { performClick, safeEval, Scope } from '../app';

import { Icon } from '../../../../shared/components/Icon';
import { cx } from '../../../../shared/styles';

export interface InnerItemProps extends PropsWithChildren {
  module: ToolbarModule;
  extraVars?: Record<string, any>;
  active?: boolean;
  clickable?: boolean;
  onWheel?: (e: React.WheelEvent) => void;
  // needed for dropdown/popup wrappers
  onClick?: (e: React.MouseEvent) => void;
  onKeydown?: (e: React.KeyboardEvent) => void;
}

interface StringToElementProps {
  text: string;
}

interface StringToElementState {
  exe_icon_path: string;
}

class StringToElement extends React.PureComponent<StringToElementProps, StringToElementState> {
  static splitter = /:([^:]+):/;

  static imgPrefix = 'IMG:';
  static iconPrefix = 'ICON:';
  static exePrefix = 'EXE:';

  static getIcon(name: string, size = 16) {
    return `[ICON:${name}:${size}]`;
  }

  static imgFromUrl(url: string, size = 16) {
    if (!url) {
      return '';
    }
    return `[IMG:${size}px:${url}]`;
  }

  static imgFromPath(path: string, size = 16) {
    if (!path) {
      return '';
    }
    return StringToElement.imgFromUrl(convertFileSrc(path), size);
  }

  static imgFromExe(exe_path: string, size = 16) {
    if (!exe_path) {
      return '';
    }
    return `[EXE:${size}px:${exe_path}]`;
  }

  constructor(props: StringToElementProps) {
    super(props);
    this.state = { exe_icon_path: LAZY_CONSTANTS.MISSING_ICON_PATH };
  }

  isImg() {
    return this.props.text.startsWith(StringToElement.imgPrefix);
  }

  isIcon() {
    return this.props.text.startsWith(StringToElement.iconPrefix);
  }

  isExe() {
    return this.props.text.startsWith(StringToElement.exePrefix);
  }

  setExeIcon(exe_path: string | null) {
    this.setState({ exe_icon_path: exe_path || LAZY_CONSTANTS.MISSING_ICON_PATH });
  }

  loadExeIconToState() {
    if (this.isExe()) {
      const [_, _size, path] = this.props.text.split(StringToElement.splitter);
      if (path) {
        invoke<string | null>('get_icon', { path })
          .then(this.setExeIcon.bind(this))
          .catch(console.error);
      }
    }
  }

  componentDidMount() {
    this.loadExeIconToState();
  }

  componentDidUpdate(prevProps: StringToElementProps) {
    if (this.props.text !== prevProps.text) {
      this.loadExeIconToState();
    }
  }

  render() {
    if (this.isExe()) {
      const [_, width] = this.props.text.split(StringToElement.splitter);
      return <img src={convertFileSrc(this.state.exe_icon_path)} style={{ width }} />;
    }

    if (this.isImg()) {
      const [_, width, url] = this.props.text.split(StringToElement.splitter);
      return <img src={url} style={{ width }} />;
    }

    if (this.isIcon()) {
      const [_, iconName, size] = this.props.text.split(':');
      return <Icon iconName={iconName || ''} size={size ? size + 'px' : undefined} />;
    }

    return <span>{this.props.text}</span>;
  }
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
  const { template, tooltip, onClick: oldOnClick, onClickV2, style, id, badge } = module;

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
      overlayClassName="ft-bar-item-tooltip"
      title={tooltip ? parseStringToElements(tooltip) : undefined}
    >
      <Reorder.Item
        {...rest}
        id={id}
        style={style}
        className={cx('ft-bar-item', {
          // onClickProp is omitted cuz it always comes via context menu dropdown wrapper
          'ft-bar-item-clickable': clickable || oldOnClick || onClickV2,
          'ft-bar-item-active': active,
        })}
        onWheel={onWheelProp}
        onKeyDown={onKeydownProp}
        onClick={(e) => {
          onClickProp?.(e);

          if (onClickV2) {
            safeEval(onClickV2, scope.current);
          }

          performClick(oldOnClick, scope.current);
        }}
        value={module}
        as="div"
        transition={{ duration: 0.15 }}
      >
        <div className="ft-bar-item-content">
          {children || elements}
          {!!badgeContent?.length && <div className="ft-bar-item-badge">{badgeContent}</div>}
        </div>
      </Reorder.Item>
    </Tooltip>
  );
}
