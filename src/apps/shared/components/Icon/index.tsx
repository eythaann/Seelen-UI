import { GetIconArgs, IconPackManager } from '@seelen-ui/lib';
import { path } from '@tauri-apps/api';
import { convertFileSrc } from '@tauri-apps/api/core';
import { UnlistenFn } from '@tauri-apps/api/event';
import React, { HTMLAttributes, ImgHTMLAttributes } from 'react';

import { cx } from '../../styles';
import InlineSVG from '../InlineSvg';
import cs from './index.module.css';

interface ReactIconProps extends HTMLAttributes<HTMLElement> {
  iconName: string;
  size?: string | number;
  color?: string;
}

/** React Icons */
export function Icon(props: ReactIconProps) {
  const { iconName, size, color, className, ...rest } = props;

  return (
    <InlineSVG
      {...rest}
      src={`../icons/${iconName}.svg`}
      className={cx('slu-icon', cs.icon, className)}
      style={{ height: size, color }}
    />
  );
}

interface FileIconProps extends GetIconArgs, Omit<ImgHTMLAttributes<HTMLImageElement>, 'src'> {
  /** if true, no missing icon will be rendered in case no icon found */
  noFallback?: boolean;
}

interface FileIconState {
  src: string | null;
}

const iconPackManager = await IconPackManager.create();
// move this to icon packs
const MISSING_ICON_SRC = convertFileSrc(await path.resolveResource('static\\icons\\missing.png'));

export class FileIcon extends React.Component<FileIconProps, FileIconState> {
  unlistener: UnlistenFn | null = null;

  constructor(props: FileIconProps) {
    super(props);
    this.updateSrc = this.updateSrc.bind(this);

    this.state = {
      src: iconPackManager.getIcon({ path: this.props.path, umid: this.props.umid }),
    };

    iconPackManager.onChange(this.updateSrc).then((unlistener) => {
      this.unlistener = unlistener;
      if (!this.state.src) {
        this.requestIconExtraction();
      }
    });
  }

  componentWillUnmount(): void {
    this.unlistener?.();
    this.unlistener = null;
  }

  componentDidUpdate(
    prevProps: Readonly<FileIconProps>,
    prevState: Readonly<FileIconState>,
    _snapshot?: any,
  ): void {
    if (this.props.path !== prevProps.path || this.props.umid !== prevProps.umid) {
      this.updateSrc();
    }

    if (prevState.src && !this.state.src) {
      this.requestIconExtraction();
    }
  }

  requestIconExtraction(): void {
    IconPackManager.extractIcon({
      path: this.props.path,
      umid: this.props.umid,
    });
  }

  updateSrc(): void {
    this.setState({
      src: iconPackManager.getIcon({ path: this.props.path, umid: this.props.umid }),
    });
  }

  render(): React.ReactNode {
    const { path: _path, umid: _umid, noFallback, ...imgProps } = this.props;

    let src = this.state.src;
    if (!src && !noFallback) {
      src = MISSING_ICON_SRC;
    }

    if (!src) {
      return null;
    }

    return <img {...imgProps} src={src} />;
  }
}
