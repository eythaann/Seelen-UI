import { exposedIcons, Icon, IconName } from '../../../shared/components/Icon';
import { ToolbarModule } from '../../../shared/schemas/Placeholders';
import { cx } from '../../../shared/styles';
import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import { Tooltip } from 'antd';
import { Reorder } from 'framer-motion';
import { cloneDeep } from 'lodash';
import { evaluate } from 'mathjs';
import React, { PropsWithChildren, useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { LAZY_CONSTANTS } from '../shared/utils/infra';

import { Selectors } from '../shared/store/app';
import { performClick, safeEval, Scope } from './app';

interface Props extends PropsWithChildren {
  module: ToolbarModule;
  extraVars?: Record<string, any>;
  active?: boolean;
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

class StringToElement extends React.Component<StringToElementProps, StringToElementState> {
  static splitter = /:([^:]+):/;

  static imgPrefix = 'IMG:';
  static iconPrefix = 'ICON:';
  static exePrefix = 'EXE:';

  static imgFromUrl(url: string, size = 16) {
    return `[IMG:${size}px:${url}]`;
  }

  static imgFromPath(path: string, size = 16) {
    return StringToElement.imgFromUrl(convertFileSrc(path), size);
  }

  static imgFromExe(exe_path: string, size = 16) {
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

  componentDidMount() {
    if (this.isExe()) {
      const [_, _size, path] = this.props.text.split(StringToElement.splitter);
      if (path) {
        invoke<string | null>('get_icon', { path })
          .then(this.setExeIcon.bind(this))
          .catch(console.error);
      }
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
      return (
        <Icon
          iconName={iconName as IconName}
          propsIcon={{ size: size ? size + 'px' : undefined }}
        />
      );
    }

    return <span>{this.props.text}</span>;
  }
}

export function ElementsFromEvaluated(content: any) {
  let text: string = content;

  if (typeof content !== 'string') {
    text = JSON.stringify(content);
  }

  const parts: string[] = text.split(/\[|\]/g).filter((part: string) => part);
  const result: React.ReactNode[] = parts.map((part: string, index: number) => {
    return <StringToElement text={part} key={index} />;
  });

  return result;
}

export function Item(props: Props) {
  const {
    extraVars,
    module,
    active,
    onClick: onClickProp,
    onKeydown: onKeydownProp,
    children,
  } = props;
  const { template, tooltip, onClick: oldOnClick, onClickV2, style, id } = module;

  const [mounted, setMounted] = React.useState(false);
  const env = useSelector(Selectors.env);
  const window = useSelector(Selectors.focused) || {
    name: 'None',
    title: 'No Window Focused',
  };

  const { t } = useTranslation();
  const scope = useRef(new Scope());

  useEffect(() => {
    scope.current.loadInvokeActions();

    scope.current.set('icon', cloneDeep(exposedIcons));
    scope.current.set('env', cloneDeep(env));
    scope.current.set('imgFromUrl', StringToElement.imgFromUrl);
    scope.current.set('imgFromPath', StringToElement.imgFromPath);
    scope.current.set('imgFromExe', StringToElement.imgFromExe);
    setMounted(true);
  }, []);

  if (!mounted) {
    return null;
  }

  scope.current.set('t', t);
  scope.current.set('window', { ...window });
  if (extraVars) {
    Object.keys(extraVars).forEach((key) => {
      scope.current.set(key, extraVars[key]);
    });
  }

  const elements = ElementsFromEvaluated(evaluate(template, scope.current));
  if (!elements.length && !children) {
    return null;
  }

  return (
    <Tooltip
      arrow={false}
      mouseLeaveDelay={0}
      overlayClassName="ft-bar-item-tooltip"
      title={tooltip ? ElementsFromEvaluated(evaluate(tooltip, scope.current)) : undefined}
    >
      <Reorder.Item
        id={id}
        style={style}
        className={cx('ft-bar-item', {
          'ft-bar-item-clickable': oldOnClick || onClickProp || onClickV2,
          'ft-bar-item-active': active,
        })}
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
        <div className="ft-bar-item-content">{children || elements}</div>
      </Reorder.Item>
    </Tooltip>
  );
}
